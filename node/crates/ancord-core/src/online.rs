//! Online klient (`spec/PROTOCOL.md` §15.12): jedno spojení s gateway odebírá schránku i všechny konverzace.
//!
//! Gateway oznámí anchorům presence, takže nové zprávy dostává klient hned. Po odpojení presence zanikne a anchor
//! příjemce vzbudí pushem. Stav (klíče, ratchet) se ukládá vždy před potvrzením (`ack`), aby pád nic neztratil.

use std::time::Duration;

use crate::encoding::{b64u_decode_array, b64u_encode};
use crate::user::{inbox_id, UserKeys};
use serde_json::{json, Value};

use crate::net::{ack_inbox, now_s, process_inbox, publish_profile, rpc, CliError, NodeSocket, Session};

/// Nejvýše tolik odběrů na jedno spojení (spec §13.3); jeden patří schránce.
pub const MAX_SUBSCRIPTIONS: usize = 32;

/// Kam online klient ukládá stav. Každé volání musí být trvalé dřív, než vrátí `Ok`.
pub trait Persist {
    fn save_keys(&mut self, keys: &UserKeys) -> Result<(), CliError>;
    fn save_conversation(&mut self, index: usize, session: &Session) -> Result<(), CliError>;
    /// Uloží novou konverzaci, která dostala index `index`.
    fn add_conversation(&mut self, index: usize, session: &Session) -> Result<(), CliError>;
}

/// Co se stalo od posledního volání.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnlineEvent {
    /// Nová konverzace (protistrana nás našla podle adresy); `watched` je `false`, když už nezbyl odběr.
    Contact { index: usize, watched: bool },
    /// Dešifrovaná a potvrzená zpráva v konverzaci `index`.
    Message { index: usize, text: Vec<u8> },
    /// Protistrana v konverzaci `index` začala nebo přestala psát.
    Typing { index: usize, on: bool },
    /// Úvodní obálky, které zatím nešlo zpracovat (profil odesílatele nenalezen); zkusí se znovu.
    Postponed(usize),
}

pub struct Online {
    pub keys: UserKeys,
    pub conversations: Vec<Session>,
    pow_bits: u32,
    /// Obtížnost razítka do vlastní schránky (stráž, spec §15.13).
    pub inbox_bits: u8,
    /// Konverzace, jejichž mailbox odebírá aktuální spojení.
    watched: Vec<bool>,
}

impl Online {
    pub fn new(keys: UserKeys, conversations: Vec<Session>, pow_bits: u32) -> Self {
        let watched = vec![false; conversations.len()];
        Self { keys, conversations, pow_bits, inbox_bits: crate::net::INBOX_POW_BITS, watched }
    }

    fn inbox(&self) -> [u8; 32] {
        inbox_id(&self.keys.pk())
    }

    /// Po připojení: zveřejní profil, přihlásí odběr schránky a konverzací a vyzvedne, co čekalo offline.
    /// Vrací konverzace, které se na spojení nevešly (víc než `MAX_SUBSCRIPTIONS − 1`).
    pub async fn attach(&mut self, ws: &mut NodeSocket, store: &mut impl Persist) -> Result<(Vec<OnlineEvent>, Vec<usize>), CliError> {
        // tajné části nových prekeys se uloží dřív, než je profil zveřejní
        if self.keys.maintain(now_s()) {
            store.save_keys(&self.keys)?;
        }
        publish_profile(ws, &self.keys, self.pow_bits).await?;
        crate::net::publish_guard(ws, &self.keys, self.inbox_bits).await?;
        self.watched = vec![false; self.conversations.len()];
        let mut ids = vec![b64u_encode(&self.inbox())];
        let mut skipped = Vec::new();
        for (i, s) in self.conversations.iter().enumerate() {
            if ids.len() < MAX_SUBSCRIPTIONS {
                ids.push(b64u_encode(&s.inbox()));
                self.watched[i] = true;
            } else {
                skipped.push(i);
            }
        }
        // nejdřív odběr (nové zprávy pak chodí samy), potom to, co čekalo
        rpc(ws, json!({"t": "subscribe", "mailbox_ids": ids})).await?;
        let mut events = self.drain_inbox(ws, store).await?;
        for i in 0..self.conversations.len() {
            if self.watched[i] {
                events.extend(self.drain(ws, store, i).await?);
            }
        }
        Ok((events, skipped))
    }

    /// Počká na další rámec od gateway a zpracuje ho. `None` po vypršení `wait` (klient má poslat `ping`).
    pub async fn next(&mut self, ws: &mut NodeSocket, store: &mut impl Persist, wait: Duration) -> Result<Option<Vec<OnlineEvent>>, CliError> {
        let Some(frame) = ws.recv_json_timeout(wait).await? else { return Ok(None) };
        self.handle(ws, store, &frame).await.map(Some)
    }

    /// Zpracuje rámec od gateway. Čekání na rámec (`recv_json`) lze přerušit, tuhle funkci ne: mezi zpracováním obálky
    /// a uložením klíčů by se jednorázový prekey ztratil a s ním i nová konverzace.
    pub async fn handle(&mut self, ws: &mut NodeSocket, store: &mut impl Persist, frame: &Value) -> Result<Vec<OnlineEvent>, CliError> {
        let Some(mailbox) = frame["mailbox_id"].as_str().and_then(b64u_decode_array::<32>) else { return Ok(Vec::new()) };
        if mailbox == self.inbox() {
            return match frame["t"].as_str() {
                Some("message") => self.drain_inbox(ws, store).await,
                _ => Ok(Vec::new()),
            };
        }
        let Some(index) = self.conversations.iter().position(|s| s.inbox() == mailbox) else { return Ok(Vec::new()) };
        match frame["t"].as_str() {
            Some("typing") => Ok(vec![OnlineEvent::Typing { index, on: frame["on"].as_bool().unwrap_or(false) }]),
            Some("message") => self.accept_pushed(ws, store, index, frame).await,
            _ => Ok(Vec::new()),
        }
    }

    /// Založí konverzaci podle adresy (spec §15.4), přihlásí odběr jejího mailboxu a vrátí její index.
    pub async fn start(&mut self, ws: &mut NodeSocket, store: &mut impl Persist, peer: &[u8; 32]) -> Result<OnlineEvent, CliError> {
        let address = b64u_encode(peer);
        if self.conversations.iter().any(|s| s.peer.as_deref() == Some(address.as_str())) {
            return Err(CliError::Session("a conversation with this address already exists".into()));
        }
        let session = crate::net::start_conversation(ws, &self.keys, peer).await?;
        let index = self.conversations.len();
        store.add_conversation(index, &session)?;
        self.conversations.push(session);
        let watched = self.watched.iter().filter(|w| **w).count() + 1 < MAX_SUBSCRIPTIONS;
        if watched {
            rpc(ws, json!({"t": "subscribe", "mailbox_id": b64u_encode(&self.conversations[index].inbox())})).await?;
        }
        self.watched.push(watched);
        Ok(OnlineEvent::Contact { index, watched })
    }

    /// Zpráva, kterou anchor přeposlal přes gateway. Duplicitu (už přijatou) ratchet odmítne a nic se nestane.
    async fn accept_pushed(&mut self, ws: &mut NodeSocket, store: &mut impl Persist, index: usize, frame: &Value) -> Result<Vec<OnlineEvent>, CliError> {
        let Some((id, ct)) = frame["id"].as_str().and_then(b64u_decode_array::<16>).zip(frame["ct"].as_str()) else { return Ok(Vec::new()) };
        let session = &mut self.conversations[index];
        let Ok(plain) = session.ratchet.open(&id, ct) else { return Ok(Vec::new()) };
        let shown = session.accept(&id, &plain);
        store.save_conversation(index, session)?;
        rpc(ws, json!({"t": "ack", "mailbox_id": b64u_encode(&session.inbox()), "ids": [b64u_encode(&id)]})).await?;
        Ok(if shown { vec![OnlineEvent::Message { index, text: plain }] } else { Vec::new() })
    }

    /// Vyzvedne všechny čekající zprávy konverzace, dešifruje je v pořadí, uloží stav a potvrdí je.
    async fn drain(&mut self, ws: &mut NodeSocket, store: &mut impl Persist, index: usize) -> Result<Vec<OnlineEvent>, CliError> {
        let mailbox = self.conversations[index].inbox();
        let reply = rpc(ws, json!({"t": "fetch", "mailbox_id": b64u_encode(&mailbox), "limit": 500})).await?;
        let session = &mut self.conversations[index];
        let mut opened = Vec::new();
        for m in reply["messages"].as_array().map_or(&[][..], Vec::as_slice) {
            let Some((id, ct)) = m["id"].as_str().and_then(b64u_decode_array::<16>).zip(m["ct"].as_str()) else { continue };
            if let Ok((n, plain)) = session.ratchet.open_numbered(&id, ct) {
                opened.push((n, id, plain));
            }
        }
        if opened.is_empty() {
            return Ok(Vec::new());
        }
        opened.sort_by_key(|(n, _, _)| *n);
        let ids: Vec<String> = opened.iter().map(|(_, id, _)| b64u_encode(id)).collect();
        opened.retain(|(_, id, plain)| session.accept(id, plain));
        store.save_conversation(index, session)?;
        rpc(ws, json!({"t": "ack", "mailbox_id": b64u_encode(&mailbox), "ids": ids})).await?;
        Ok(opened.into_iter().map(|(_, _, text)| OnlineEvent::Message { index, text }).collect())
    }

    /// Zpracuje úvodní obálky ve schránce: založí konverzace, přihlásí jejich odběr a vyzvedne, co v nich už čeká.
    async fn drain_inbox(&mut self, ws: &mut NodeSocket, store: &mut impl Persist) -> Result<Vec<OnlineEvent>, CliError> {
        let batch = process_inbox(ws, &mut self.keys).await?;
        let mut events = Vec::new();
        let mut added = Vec::new();
        for a in batch.accepted {
            // s protistranou už konverzace existuje: stav ratchetu se nikdy nepřepisuje
            if self.conversations.iter().any(|s| s.peer == a.session.peer) {
                continue;
            }
            let index = self.conversations.len();
            store.add_conversation(index, &a.session)?;
            self.conversations.push(a.session);
            self.watched.push(false);
            added.push(index);
        }
        // klíče bez použitých jednorázových prekeys musí být uložené dřív, než obálky zmizí ze schránky
        store.save_keys(&self.keys)?;
        ack_inbox(ws, &self.keys, &batch.done).await?;
        if batch.postponed > 0 {
            events.push(OnlineEvent::Postponed(batch.postponed));
        }
        if added.is_empty() {
            return Ok(events);
        }
        // spotřebované jednorázové prekeys: nový profil, aby je nikdo nepoužil znovu
        self.keys.maintain(now_s());
        store.save_keys(&self.keys)?;
        publish_profile(ws, &self.keys, self.pow_bits).await?;
        for index in added {
            let watched = self.watched.iter().filter(|w| **w).count() + 1 < MAX_SUBSCRIPTIONS;
            if watched {
                rpc(ws, json!({"t": "subscribe", "mailbox_id": b64u_encode(&self.conversations[index].inbox())})).await?;
                self.watched[index] = true;
            }
            events.push(OnlineEvent::Contact { index, watched });
            if watched {
                events.extend(self.drain(ws, store, index).await?);
            }
        }
        Ok(events)
    }

    /// Pošle keepalive do konverzací, kde přijatá zpráva čeká na odpověď déle než `KEEPALIVE_DELAY_S` (spec §13.2).
    /// Volá se pravidelně, např. při nečinnosti spojení. Vrací počet odeslaných.
    pub async fn keepalives(&mut self, ws: &mut NodeSocket, store: &mut impl Persist) -> Result<usize, CliError> {
        let mut sent = 0;
        for index in 0..self.conversations.len() {
            let session = &mut self.conversations[index];
            if crate::net::keepalive_if_due(ws, session, crate::net::KEEPALIVE_DELAY_S, |s| store.save_conversation(index, s)).await? {
                sent += 1;
            }
        }
        Ok(sent)
    }

    /// Zašifruje a odešle zprávu do konverzace `index`. Stav ratchetu se uloží před odesláním.
    pub async fn send(&mut self, ws: &mut NodeSocket, store: &mut impl Persist, index: usize, text: &[u8]) -> Result<[u8; 16], CliError> {
        let session = self.conversations.get_mut(index).ok_or_else(|| CliError::Session("no such conversation".into()))?;
        let prepared = session.prepare(text)?;
        store.save_conversation(index, session)?;
        crate::net::transmit(ws, &prepared).await?;
        Ok(prepared.id)
    }
}

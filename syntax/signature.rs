use crate::syntax::{FnKind, Receiver, Signature};

impl Signature {
    pub fn receiver(&self) -> Option<&Receiver> {
        match &self.kind {
            FnKind::Method(receiver) => Some(receiver),
            FnKind::Assoc(_) | FnKind::Free => None,
        }
    }

    pub fn receiver_mut(&mut self) -> Option<&mut Receiver> {
        match &mut self.kind {
            FnKind::Method(receiver) => Some(receiver),
            FnKind::Assoc(_) | FnKind::Free => None,
        }
    }
}

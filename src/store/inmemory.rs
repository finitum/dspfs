use crate::error::DspfsError;
use crate::store::Store;
use crate::user::PublicUser;
use ring::pkcs8::Document;
use ring::signature::Ed25519KeyPair;

pub struct InMemory {
    user: Option<PublicUser>,
    signing_key: Option<Document>,
}

impl Default for InMemory {
    fn default() -> InMemory {
        Self {
            user: None,
            signing_key: None,
        }
    }
}

impl Store for InMemory {
    fn set_self_user(&mut self, user: PublicUser) {
        self.user = Some(user)
    }

    fn get_self_user(&self) -> &Option<PublicUser> {
        &self.user
    }

    fn set_signing_key(&mut self, key: Document) {
        self.signing_key = Some(key)
    }

    fn get_signing_key(self) -> Option<Result<Ed25519KeyPair, DspfsError>> {
        self.signing_key
            .map(|document| Ok(Ed25519KeyPair::from_pkcs8(document.as_ref())?))
    }
}

mod private;
mod public;

pub use private::PrivateUser;
pub use public::PublicUser;
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};

#[derive(serde::Serialize, serde::Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct PublicKey(pub(self) Vec<u8>);

impl PublicKey {
    pub fn ring(&self) -> UnparsedPublicKey<&[u8]> {
        UnparsedPublicKey::new(&ED25519, &self.0)

    }
}

impl From<Vec<u8>> for PublicKey {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<&Ed25519KeyPair> for PublicKey {
    fn from(k: &Ed25519KeyPair) -> Self {
        Self(k.public_key().as_ref().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::user::PrivateUser;
    use ring::signature::{Ed25519KeyPair, KeyPair};

    #[test]
    fn test_new_user() {
        let (u1, skd1) = PrivateUser::new("Test1").unwrap();
        let (u2, skd2) = PrivateUser::new("Test2").unwrap();

        assert_ne!(u1.get_public_key(), u2.get_public_key());

        let kp1 = Ed25519KeyPair::from_pkcs8(skd1.as_ref()).unwrap();
        let kp2 = Ed25519KeyPair::from_pkcs8(skd2.as_ref()).unwrap();

        assert_eq!(kp1.public_key().as_ref(), u1.get_public_key().0.as_slice());
        assert_eq!(kp2.public_key().as_ref(), u2.get_public_key().0.as_slice());
    }
}

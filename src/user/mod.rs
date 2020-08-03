mod private;
mod public;

use anyhow::{Context, Result};
pub use private::PrivateUser;
pub use public::PublicUser;
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use serde::export::TryFrom;
use zerocopy::{AsBytes, LayoutVerified};

#[derive(serde::Serialize, serde::Deserialize, Clone, Eq, PartialEq, Debug, AsBytes, Ord, PartialOrd)]
#[repr(packed)]
pub struct PublicKey(pub(self) [u8; 32]);

impl PublicKey {
    pub fn ring(&self) -> UnparsedPublicKey<&[u8]> {
        UnparsedPublicKey::new(&ED25519, &self.0)
    }
}

impl TryFrom<Vec<u8>> for PublicKey {
    type Error = anyhow::Error;

    fn try_from(v: Vec<u8>) -> Result<Self> {
        let a = *LayoutVerified::<_, [u8; 32]>::new(v.as_ref())
            .context("Public key is not 32 bytes")?;

        Ok(Self(a))
    }
}

impl TryFrom<&Ed25519KeyPair> for PublicKey {
    type Error = anyhow::Error;

    fn try_from(k: &Ed25519KeyPair) -> Result<Self> {
        let a = *LayoutVerified::<_, [u8; 32]>::new(k.public_key().as_ref())
            .context("Public key is not 32 bytes")?;

        Ok(Self(a))
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

        assert_eq!(kp1.public_key().as_ref(), u1.get_public_key().0.as_ref());
        assert_eq!(kp2.public_key().as_ref(), u2.get_public_key().0.as_ref());
    }
}

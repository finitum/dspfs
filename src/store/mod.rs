use crate::error::DspfsError;
use crate::user::PublicUser;
use ring::pkcs8;
use ring::signature::Ed25519KeyPair;

pub mod inmemory;

pub trait Store: Default {
    /// Saves the private user in the store
    fn set_self_user(&mut self, user: PublicUser);

    /// Returns the private user saved in the store or None if it doesn't exist
    fn get_self_user(&self) -> &Option<PublicUser>;

    /// Savestheprivatekeyinsideofthestore
    fn set_signing_key(&mut self, key: pkcs8::Document);

    ///returns the ed25119 key pair based on the store private key
    fn get_signing_key(self) -> Option<Result<Ed25519KeyPair, DspfsError>>;
}

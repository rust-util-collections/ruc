use super::origin;
use crate::{crypto::codec::base64, *};
use std::fmt;

pub use ed25519_zebra::SigningKey as RawSignKey;
pub use ed25519_zebra::VerificationKey as RawVerifyKey;

#[cfg_attr(feature = "SerDe", derive(serde::Deserialize, serde::Serialize))]
pub struct SignKey(String);

#[cfg_attr(feature = "SerDe", derive(serde::Deserialize, serde::Serialize))]
pub struct VerifyKey(String);

#[cfg_attr(feature = "SerDe", derive(serde::Deserialize, serde::Serialize))]
pub struct Sig(String);

// <signing key + verify key> in base64 format
pub fn create_keypair() -> (SignKey, VerifyKey) {
    let (sk, vk) = origin::create_keypair();
    (sk.into(), vk.into())
}

impl SignKey {
    pub fn sign(&self, msg: &[u8]) -> Result<Sig> {
        let sk = base64::decode(&self.0).c(d!())?;
        let sk = RawSignKey::try_from(sk.as_slice()).c(d!())?;
        let sig = base64::encode(sk.sign(msg).to_bytes());
        Ok(Sig(sig))
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for SignKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for SignKey {
    type Error = Box<dyn RucError>;
    fn try_from(s: String) -> Result<Self> {
        let sk = base64::decode(&s).c(d!())?;
        RawSignKey::try_from(sk.as_slice()).c(d!())?;
        Ok(Self(s))
    }
}

impl TryFrom<&str> for SignKey {
    type Error = Box<dyn RucError>;
    fn try_from(s: &str) -> Result<Self> {
        let sk = base64::decode(s).c(d!())?;
        RawSignKey::try_from(sk.as_slice()).c(d!())?;
        Ok(Self(s.to_owned()))
    }
}

impl TryFrom<&SignKey> for RawSignKey {
    type Error = Box<dyn RucError>;
    fn try_from(k: &SignKey) -> Result<Self> {
        let sk = base64::decode(&k.0).c(d!())?;
        RawSignKey::try_from(sk.as_slice()).c(d!())
    }
}

impl TryFrom<SignKey> for RawSignKey {
    type Error = Box<dyn RucError>;
    fn try_from(k: SignKey) -> Result<Self> {
        Self::try_from(&k)
    }
}

impl From<&RawSignKey> for SignKey {
    fn from(sk: &RawSignKey) -> SignKey {
        SignKey(base64::encode(sk.as_ref()))
    }
}

impl From<RawSignKey> for SignKey {
    fn from(sk: RawSignKey) -> SignKey {
        SignKey::from(&sk)
    }
}

impl VerifyKey {
    pub fn verify(&self, sig: &Sig, msg: &[u8]) -> Result<()> {
        let vk = base64::decode(&self.0).c(d!())?;
        let vk = RawVerifyKey::try_from(vk.as_slice()).c(d!())?;
        verify_by_raw_vk(&vk, sig, msg).c(d!())
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for VerifyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for VerifyKey {
    type Error = Box<dyn RucError>;
    fn try_from(s: String) -> Result<Self> {
        let vk = base64::decode(&s).c(d!())?;
        RawVerifyKey::try_from(vk.as_slice()).c(d!())?;
        Ok(Self(s))
    }
}

impl TryFrom<&str> for VerifyKey {
    type Error = Box<dyn RucError>;
    fn try_from(s: &str) -> Result<Self> {
        let vk = base64::decode(s).c(d!())?;
        RawVerifyKey::try_from(vk.as_slice()).c(d!())?;
        Ok(Self(s.to_owned()))
    }
}

impl TryFrom<&VerifyKey> for RawVerifyKey {
    type Error = Box<dyn RucError>;
    fn try_from(k: &VerifyKey) -> Result<Self> {
        let vk = base64::decode(&k.0).c(d!())?;
        RawVerifyKey::try_from(vk.as_slice()).c(d!())
    }
}

impl TryFrom<VerifyKey> for RawVerifyKey {
    type Error = Box<dyn RucError>;
    fn try_from(k: VerifyKey) -> Result<Self> {
        Self::try_from(&k)
    }
}

impl From<&RawVerifyKey> for VerifyKey {
    fn from(vk: &RawVerifyKey) -> VerifyKey {
        VerifyKey(base64::encode(vk.as_ref()))
    }
}

impl From<RawVerifyKey> for VerifyKey {
    fn from(vk: RawVerifyKey) -> VerifyKey {
        VerifyKey::from(&vk)
    }
}

pub fn verify_by_raw_vk(
    vk: &RawVerifyKey,
    sig: &Sig,
    msg: &[u8],
) -> Result<()> {
    let sig = base64::decode(&sig.0).c(d!())?;
    let sig = ed25519_zebra::Signature::try_from(sig.as_slice()).c(d!())?;
    vk.verify(&sig, msg).c(d!())
}

impl Sig {
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for Sig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Sig {
    type Error = Box<dyn RucError>;
    fn try_from(s: String) -> Result<Self> {
        let sig = base64::decode(&s).c(d!())?;
        ed25519_zebra::Signature::try_from(sig.as_slice()).c(d!())?;
        Ok(Self(s))
    }
}

impl TryFrom<&str> for Sig {
    type Error = Box<dyn RucError>;
    fn try_from(s: &str) -> Result<Self> {
        let sig = base64::decode(s).c(d!())?;
        ed25519_zebra::Signature::try_from(sig.as_slice()).c(d!())?;
        Ok(Self(s.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sign_verify() {
        let msg = b"aljflajfljaljfl";
        let (sk, vk) = create_keypair();

        let sig = sk.sign(msg).unwrap();
        let sig = sig.into_string();
        let sig = Sig::try_from(sig).unwrap();

        assert!(vk.verify(&sig, msg).is_ok());

        let msg_fake = b"00000000000000000";
        assert!(vk.verify(&sig, msg_fake).is_err());
    }

    #[test]
    fn parse_key() {
        assert!(SignKey::try_from(";akjflkjafj".to_owned()).is_err());
        assert!(VerifyKey::try_from(";akjflkjafj".to_owned()).is_err());

        let (sk, vk) = create_keypair();
        assert!(SignKey::try_from(sk.to_string()).is_ok());
        assert!(VerifyKey::try_from(vk.to_string()).is_ok());
    }
}

//! Secret with additional network identifier and format type

use address::detect_checksum;
use std::fmt;
use std::str::FromStr;
use secp256k1::key;
use secp256k1::Message as SecpMessage;
use hex::ToHex;
use base58::{ToBase58, FromBase58};
use crypto::{checksum, ChecksumType};
use hash::H520;
use {Secret, DisplayLayout, Error, Message, Signature, CompactSignature, SECP256K1};

/// Secret with additional network prefix and format type
#[derive(Default, PartialEq)]
pub struct Private {
	/// The network prefix on which this key should be used.
	pub prefix: u8,
	/// ECDSA key.
	pub secret: Secret,
	/// True if this private key represents a compressed address.
	pub compressed: bool,
	/// checksum type
	pub checksum_type: ChecksumType,
}

impl Private {
	pub fn sign(&self, message: &Message) -> Result<Signature, Error> {
		let context = &SECP256K1;
		let secret = try!(key::SecretKey::from_slice(context, &*self.secret));
		let message = try!(SecpMessage::from_slice(&**message));
		let signature = try!(context.sign(&message, &secret));
		let data = signature.serialize_der(context);
		Ok(data.into())
	}

	pub fn sign_compact(&self, message: &Message) -> Result<CompactSignature, Error> {
		let context = &SECP256K1;
		let secret = try!(key::SecretKey::from_slice(context, &*self.secret));
		let message = try!(SecpMessage::from_slice(&**message));
		let signature = try!(context.sign_recoverable(&message, &secret));
		let (recovery_id, data) = signature.serialize_compact(context);
		let recovery_id = recovery_id.to_i32() as u8;
		let mut signature = H520::default();
		signature[1..65].copy_from_slice(&data[0..64]);
		if self.compressed {
			signature[0] = 27 + recovery_id + 4;
		} else {
			signature[0] = 27 + recovery_id;
		}
		Ok(signature.into())
	}
}

impl DisplayLayout for Private {
	type Target = Vec<u8>;

	fn layout(&self) -> Self::Target {
		let mut result = vec![];
		result.push(self.prefix);
		result.extend(&*self.secret);
		if self.compressed {
			result.push(1);
		}
		let cs = checksum(&result, &self.checksum_type);
		result.extend_from_slice(&*cs);
		result
	}

	fn from_layout(data: &[u8]) -> Result<Self, Error> where Self: Sized {
		let compressed = match data.len() {
			37 => false,
			38 => true,
			_ => return Err(Error::InvalidPrivate),
		};

		if compressed && data[data.len() - 5] != 1 {
			return Err(Error::InvalidPrivate);
		}

		let sum_type = detect_checksum(&data[0..data.len() - 4], &data[data.len() - 4..])?;
		let prefix = data[0];

		let mut secret = Secret::default();
		secret.copy_from_slice(&data[1..33]);

		let private = Private {
			prefix,
			secret,
			compressed,
            checksum_type: sum_type,
		};

		Ok(private)
	}
}

impl fmt::Debug for Private {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		try!(writeln!(f, "prefix: {:?}", self.prefix));
		try!(writeln!(f, "secret: {}", self.secret.to_hex()));
		writeln!(f, "compressed: {}", self.compressed)
	}
}

impl fmt::Display for Private {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.layout().to_base58().fmt(f)
	}
}

impl FromStr for Private {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Error> where Self: Sized {
		let hex = try!(s.from_base58().map_err(|_| Error::InvalidPrivate));
		Private::from_layout(&hex)
	}
}

impl From<&'static str> for Private {
	fn from(s: &'static str) -> Self {
		s.parse().unwrap()
	}
}

#[cfg(test)]
mod tests {
	use hash::H256;
	use super::{ChecksumType, Private};

	#[test]
	fn test_private_to_string() {
		let private = Private {
			prefix: 128,
			secret: H256::from_reversed_str("063377054c25f98bc538ac8dd2cf9064dd5d253a725ece0628a34e2f84803bd5"),
			compressed: false,
			checksum_type: ChecksumType::DSHA256,
		};

		assert_eq!("5KSCKP8NUyBZPCCQusxRwgmz9sfvJQEgbGukmmHepWw5Bzp95mu".to_owned(), private.to_string());
	}

	#[test]
	fn test_private_to_string_kmd() {
		let private = Private {
			prefix: 188,
			secret: H256::from_reversed_str("063377054c25f98bc538ac8dd2cf9064dd5d253a725ece0628a34e2f84803bd5"),
			compressed: true,
			checksum_type: ChecksumType::DSHA256,
		};

		assert_eq!("UwA3FpHWKfwrQ1DTiwbErpEnCEhvLuq1WnbfmqGBPSLNNvXtzYd5".to_owned(), private.to_string());
	}

    #[test]
	fn test_private_to_string_zec_testnet() {
		let private = Private {
			prefix: 239,
			secret: H256::from_reversed_str("063377054c25f98bc538ac8dd2cf9064dd5d253a725ece0628a34e2f84803bd5"),
			compressed: true,
			checksum_type: ChecksumType::DSHA256,
		};

		assert_eq!("cUjCR3fPFWfs6PtdvoinTh4ctPxBvFf5pKNKJzw1RqmfjogL7GuU".to_owned(), private.to_string());
	}

	#[test]
	fn test_private_from_str() {
		let private = Private {
			prefix: 128,
			secret: H256::from_reversed_str("063377054c25f98bc538ac8dd2cf9064dd5d253a725ece0628a34e2f84803bd5"),
			compressed: false,
			checksum_type: ChecksumType::DSHA256,
		};

		assert_eq!(private, "5KSCKP8NUyBZPCCQusxRwgmz9sfvJQEgbGukmmHepWw5Bzp95mu".into());
	}

	#[test]
	fn test_private_from_str_kmd() {
		let private = Private {
			prefix: 188,
			secret: H256::from_reversed_str("063377054c25f98bc538ac8dd2cf9064dd5d253a725ece0628a34e2f84803bd5"),
			compressed: true,
			checksum_type: ChecksumType::DSHA256,
		};

		assert_eq!(private, "UwA3FpHWKfwrQ1DTiwbErpEnCEhvLuq1WnbfmqGBPSLNNvXtzYd5".into());
	}

    #[test]
	fn test_private_from_str_zec_testnet() {
		let private = Private {
			prefix: 239,
			secret: H256::from_reversed_str("063377054c25f98bc538ac8dd2cf9064dd5d253a725ece0628a34e2f84803bd5"),
			compressed: true,
			checksum_type: ChecksumType::DSHA256,
		};

		assert_eq!(private, "cUjCR3fPFWfs6PtdvoinTh4ctPxBvFf5pKNKJzw1RqmfjogL7GuU".into());
	}

	#[test]
	fn test_private_from_str_grs() {
		let private = Private {
			prefix: 128,
			secret: H256::from_reversed_str("cbc8853bd3617a5fcecfcc97f4a68853481657fc575cf85e04a64a2d1a78f974"),
			compressed: true,
			checksum_type: ChecksumType::DGROESTL512,
		};

		assert_eq!(private, "L196QUb5fAcBVvZizvx66ABsU7iVTS4iAz15YEgB8QWY35KfD6ox".into());
		assert_eq!(private.to_string(), "L196QUb5fAcBVvZizvx66ABsU7iVTS4iAz15YEgB8QWY35KfD6ox".to_owned());
	}

	#[test]
	fn test_private_from_str_smart_cash() {
		let private = Private {
			prefix: 191,
			secret: H256::from_reversed_str("48688b0cd9440864b95916f53d6e06cdab5f50dc3abfa74b5c6a176620daa302"),
			compressed: true,
			checksum_type: ChecksumType::KECCAK256,
		};

		assert_eq!(private, "VFqZrZNzkJEk29Kzp87J7eXDuQFMh1UsqYcMmi9bfdAZ522nz1mv".into());
		assert_eq!(private.to_string(), "VFqZrZNzkJEk29Kzp87J7eXDuQFMh1UsqYcMmi9bfdAZ522nz1mv".to_owned());
	}
}

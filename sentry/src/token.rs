use std::{
	collections::HashMap,
	convert::TryFrom,
	io::Result as IoResult,
};

use asap::{
	generator::Generator,
	validator::{Validator, ValidatorBuilder},
	claims::{Aud, ExtraClaims},
};

#[cfg(feature = "alloc")]
use alloc;
use log;
use magic_crypt::MagicCrypt;
use once_cell::sync::OnceCell;
use serde::{Serialize, Deserialize};
use crate::error::Error;

// Info: Use .der format for PEM key encryption
//
// Steps to use ASAP API
//
// get_validator() method builds a `Resource_Server` which will fetch `KID`,
// verify claims and signature.
//
// See [Validator, ValidatorBuilder] ASAP trait definition for more info.
//
// You'll need to setup environment variables for the validator and the generator.
// The generator holds the header which is RS256 algorithm, private key, claims
// and extra-claims.
//
// Set the "aud" claim identifier to access token generation. The issuer
// and the resource server have to mutually agree on this identifier.  The
// details of how they reach agreement is not covered in the [Atlassian S2S Auth
// protocol specification](https://s2sauth.bitbucket.io/spec/)
//
// As the aud consensus is open for implementation; Sentry token implements a
// 256-bit DES encrypted client identifier, which converts client data string
// as `aud`.
//
// Assert MASTER_ASAP_KEY.get() is empty before initialzing thread_safe_key.
// Sentry token uses the key in a Lazy sync non-copy type, similar to a
// lazy_static! but without using any macro's.
//
// To validate tokens be sure to use a keyserver. Future developments will
// include a custom async HTTP client with async/await.
//
// Reference: https://github.com/rustasync/surf

/// Private key used to sign tokens.
const PKEY: &[u8] = include_bytes!("../support/keys/sessions01/1569901546-private.der");
/// Name of the issuer for the token generating service.
const ISS: &'static str = "sessions";
/// client data used for audience_identifier obfuscation.
const AUD: &'static str = "email@example.com";
/// Path of the public key.  It will be consumed by a keyserver.
const KID: &'static str = "sessions01/1569901546-public.der";

/// Token lifespans
const REFRESH_LIFESPAN: i64 = 15 * 60;
const NORMAL_LIFESPAN: i64 = 60 * 60;

/// Master key will be consumed by the `aud` magic_crypt encrypt method
static MASTER_ASAP_KEY: OnceCell<String> = OnceCell::new();

/// A thread-safe cell which can be written to only once
pub fn init_thread_safe_key() {
	std::thread::spawn(|| {
		let _value: String = MASTER_ASAP_KEY.get_or_init(|| {
			let file_path = std::env::var("MASTER_ASAP_KEY")
				.unwrap_or("./warden.key".to_owned());

			log::debug!("Using `aud` obfuscator file {}", file_path);

			let aud_key: Vec<u8> = std::fs::read(&file_path).unwrap();

			log::debug!( "Using `aud` signer key of {} bits", aud_key.len() * 8);

			std::str::from_utf8(aud_key.as_slice()).unwrap().to_string()

		}).to_owned();
	}).join().ok().expect("Could not join a thread");
}

/// TokenType enumerates the type of Token: [Normal or Refresh]
#[derive(Debug, Serialize, Deserialize)]
pub enum TokenType {
	/// Normal token is valid for 60 minutes
	Normal,
	/// Refresh token is valid for 15 minutes
	Refresh,
}

/// From &str trait implementation
impl From<TokenType> for &'static str {
	fn from(original: TokenType) -> &'static str {
		use self::TokenType::*;

		match original {
			Normal => "Normal",
			Refresh => "Refresh",
		}
	}
}

/// Convert From enum to static string. Allows the use of TokenType variant
/// debug as such:
///
/// i.e.
/// println!("{}", Into::<&str>::into(TokenType::Refresh));
///
/// $ Refresh
impl <'a> TryFrom<&'a str> for TokenType {
	type Error = &'a str;
	fn try_from(value: &'a str) -> Result<Self, &'a str> {
		use self::TokenType::*;

		let normal: &'static str = Normal.into();
		let refresh: &'static str = Refresh.into();

		match value {
			x if x == normal => Ok(Normal),
			x if x == refresh => Ok(Refresh),
			_ => Err("Error while converting TokenType value to static string."),
		}
	}
}

/// get_validator() is a constructor method for ValidatorBuilder.
/// Incoming ASAP tokens must include resource server audience identifier in
/// their `aud` claim in order for a token to be valid.
pub fn get_validator(keyserver_uri: &str) -> ValidatorBuilder {
	let audience_identifier = encrypt_aud_to_base64(AUD);
	let resource_server_audience = String::from(audience_identifier);
	Validator::builder(String::from(keyserver_uri), resource_server_audience)
}

/// Generator builder for ASAP Claims.
fn generator_build() -> Generator {
	Generator::new(
		ISS.to_string(),
		KID.to_string(),
		PKEY.to_vec(),
		)
}

/// generate_token() takes in client_data which will be used in the 'aud'
/// claims, and TokenType variont of 'Normal' or 'Refresh'.  The method
/// returns an ASAP token with extra claims.
///
/// The token will have different lifespans depending on the TokenType variant.
pub fn generate_token(token_type: TokenType, client_data: &str)
	-> Result<String, Error>
{
	let mut generator = generator_build();
	match token_type {
		TokenType::Normal => {
			let _ = generator.set_max_lifespan(NORMAL_LIFESPAN);
			let normal_token = generator
				.token(
					default_aud(client_data),
					set_token_type(TokenType::Normal)
				)?;
			Ok(normal_token)
		},
		TokenType::Refresh => {
			let _ = generator.set_max_lifespan(REFRESH_LIFESPAN);
			let refresh_token = generator
				.token(
					default_aud(client_data),
					set_token_type(TokenType::Refresh)
				)?;
			Ok(refresh_token)
		},
	}
}

/// Encrypts the client_data to AES 256-bit, encoded as base64.
pub fn encrypt_aud_to_base64(client_data: &str) -> String {
	let key: Option<&String> = MASTER_ASAP_KEY.get();
	let mut secret: MagicCrypt = new_magic_crypt!(key.unwrap().as_str(), 256);
	let aud_claims = secret.encrypt_str_to_base64(client_data);
	log::info!("encrpted aud claims field: {}", aud_claims);

	aud_claims.to_string()
}

/// decrypt_aud() takes in AES 256-bit base64 encoded string and decrypts it.
pub fn decrypt_aud(audience_identifier: &str) -> Result<String, Error> {
	let key: Option<&String> = MASTER_ASAP_KEY.get();
	let mut secret: MagicCrypt = new_magic_crypt!(key.unwrap().as_str(), 256);
	let raw =  secret.decrypt_base64_to_string(audience_identifier).unwrap();
	Ok(raw)
}

/// Converts client_data to audience server identifier for generator consumption
fn default_aud(client_data: &str) -> Aud {
	let audience_identifier = Aud::One(encrypt_aud_to_base64(client_data));
	audience_identifier
}

/// type_of_token() is a helper method to include ExtraClaims hashMap of TokenType.
///
/// ASAP tokens have a hard limit of 3600 as their lifespan. This limitation
/// puts a halt on a new generator implementation, which intended on
/// overriding the 'exp' field and setting an ExtraClaim of TokenType.
fn set_token_type(token_type: TokenType) -> Option<ExtraClaims> {
	let normal = TokenType::Normal;
	let refresh = TokenType::Refresh;

	let mut extra_claims = HashMap::new();
	match token_type {
		TokenType::Normal => {
			extra_claims.insert("TokenType".to_string(), json!(
					Into::<&str>::into(normal)));
		},

		TokenType::Refresh => {
			extra_claims.insert("TokenType".to_string(), json!(
					Into::<&str>::into(refresh)));
		},
	}
	Some(extra_claims)
}

/// aud_from_json() extracts the inner member of Aud enum variant.
/// Its purpose is to get the `aud` variant from a token payload.
///
/// This method is credited to @Sébastien Renauld, over at StackOverflow.
/// Thank you for your help and patience.
pub fn aud_from_json(data: &asap::claims::Aud) -> IoResult<String> {
	match data {
		Aud::One(audience) => Ok(audience.clone()),
		Aud::Many(audiences) => audiences
			.last()
			.ok_or(
				std::io::Error::new(
					std::io::ErrorKind::NotFound, "No audience found")
			).map(|r| r.clone())
	}
}

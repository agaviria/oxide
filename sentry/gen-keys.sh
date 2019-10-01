# This script is used to generate a simple private/public key-pair in `der` format.
function gen_keys() {
  sessions=$1;
  key_id=$2;

  mkdir -p "support/keys/$sessions";
  openssl genrsa -out "support/keys/$sessions/$key_id-private.pem" 2048;

  openssl rsa -in "support/keys/$sessions/$key_id-private.pem" -outform DER -out "support/keys/$sessions/$key_id-private.der";
  openssl rsa -in "support/keys/$sessions/$key_id-private.der" -inform DER -RSAPublicKey_out -outform DER -out "support/keys/$sessions/$key_id-public.der";
}

gen_keys "sessions01" "$(date +%s)"
gen_keys "sessions02" "$(date +%s)"

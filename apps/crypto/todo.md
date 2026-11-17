# ChaCha20

- [ ] [Quarter Round](https://www.rfc-editor.org/info/rfc8439/#section-2.1)
- [ ] [Block Function](https://www.rfc-editor.org/info/rfc8439/#section-2.3)
- [ ] [Encryption and Decryption](https://www.rfc-editor.org/info/rfc8439/#section-2.4)

# Poly1305

- [ ] [Poly1305](https://www.rfc-editor.org/info/rfc8439/#section-2.6)
- [ ] [ChaCha20-Poly1305 AEAD](https://www.rfc-editor.org/info/rfc8439/#section-2.8)

# X25519

- [ ] [X25519](https://datatracker.ietf.org/doc/html/rfc7748#section-5)
- [ ] [Curve25519](https://datatracker.ietf.org/doc/html/rfc7748#section-6.1)

# Extras

- [ ] [SHA256](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf)
- [ ] [HKDF](https://www.rfc-editor.org/info/rfc5869)
- [ ] [HKDF-SHA256](https://www.rfc-editor.org/rfc/rfc9709.html#name-use-of-hkdf-with-sha-256-to)
- [ ] [HChaCha20](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha-01#section-2.2)
- [ ] [XChaCha20](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha-01#section-2.3)
- [ ] [XChaCha20-Poly1305 AEAD](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha-01#section-2)

# Pipeline

## Encryption

1. Generate random Data-Encryption Key (DEK) and nonce
2. Encrypt data using XChaCha20-Poly1305 with the DEK and nonce
3. Generate ephemeral key pair and shared secret with X25519
4. Derive Key-Encryption Key (KEK) from shared secret with HKDF-SHA256 (requires generating random HKDF salt)
5. Encrypt DEK using XChaCha20-Poly1305 with the KEK and a new nonce
6. Send ephemeral public key, HKDF salt, data nonce, ciphertext, DEK nonce, and encrypted DEK

## Decryption

1. Calculate shared secret using X25519 with private key and ephemeral public key
2. Derive Key-Encryption Key (KEK) from shared secret with HKDF-SHA256 and HKDF salt
3. Decrypt and authenticate DEK using XChaCha20-Poly1305 with DEK nonce and KEK
4. Decrypt and authenticate data using XChaCha20-Poly1305 with data nonce and DEK

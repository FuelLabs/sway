# Signatures, Addresses and EC-Recover

## Steps

1. __Generate a private key:__ 256 random bits

2. __Generate a public key:__ Multiply the private key by the elliptic curve generator point to get the public key. The public key is a point on the elliptic curve and has x and y coordinates (use sepc256k1).

3. __Concatenate the x and y coordinates of public key__
 then compute a Sha256 hash.

4. The resulting 32 bytes is the address !
consider using `ethers_core::types::H256` for now:
(Fixed-size uninterpreted hash type with 32 bytes (256 bits) size.)

## Test Data

using <https://learnmeabitcoin.com/technical/mnemonic>

### Entropy (256 bits)

1110110011111001011110101001111110011000100000011111000101010110000001000011011110000010010101111100100101110100110000010001111010011010011010011100010101011101101111111010010001110100111011111010001101110110000011011001001011100000101111110001111101110100

### checksum

00011111

### Mnemonic

undo slim pony country business prison awkward utility fit entry core diamond pledge tired ivory virus insane laptop talk brass come garbage lava loop

### Seed

42faf7c1630a3826c2a141ee1872b76dc47cc27df58b9a8fbd8ae0eb1e5a65c88f8978ca481a4854c6f1f49983b054137a4a2fa9940bf92189814215e612f50d

 Using <https://iancoleman.io/bip39/#english>

mnemonic:
live body box comfort verify boring grid another exchange perfect legend behind

<!-- Bip39 Seed: 3c68107c97c3624b51b9e3f9bb1d1f337f4fe2a9b1bcba56683003227f809005a069600e27124ab78b3fbb26d3806fd0532842f33afbab0720213b716f6ca8c6 -->

<!-- Bip32 Root Key:
xprv9s21ZrQH143K4KcJkDmrFoF6NvPgnY17Qe9dMNoQVepPQiDMq99p2PgDhRdBPccMzJ4uoR7YgeXwZkgLMqn35FFmgU2dkEJfjeUJrtN461A -->

Derivation Path: Bip44
Account Extended Private Key: xprv9ypA8uNbCG7XU5rJwJcZRLf5KvaT5dWzsn67nHNEXkXisqS4NFFbcTqJiwDZqbDmHTpJXcqnT6DKCaNhwAZejvBWPUCJZnqA4TMWBjUQ89x

Account Extended Public Key: xpub6CoWYQuV2dfpgZvn3L9ZnUbosxQwV6ErF11iafmr664hkdmCunZrAG9naBbaPeiMtsq2KqTzZR6LXwmvGWRfEBZ4Xv5tuNRRJL1WyPnP8TD

Address: 2da5351f422d4329b01e66a5799c86c14e0fd8d3dcb229e65799af4554df5c6a

- Sha256(Account Extended Public Key)

BIP32 Extended Private Key: xprv9zjE1KYs8PWqJvcmi3RBKWMDWoq7kdx3i7DspYp22F3tB3P8PP6zSVwscwav9f9EuWoUFxoVAwP7MvntC8ZWjZAbuAkFDmnHLc7bL2HZgxS

BIP32 Extended Public Key: xpub6DiaQq5kxm58XQhEp4xBgeHx4qfcA6fu5L9UcwDdaaas3qiGvvREzJGMUF2aNLLAmPZAHn9DJ3E7BEUMQSgbdXNzHstcFE7uMURznT1eCTs

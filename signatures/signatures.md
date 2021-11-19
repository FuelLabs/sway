# Signatures, Addresses and EC-Recover

## Steps

1. __Generate a private key:__

256 random bits

2. __Generate a public key:__

Multiply the private key by the elliptic curve generator point to get the public key. The public key is a point on the elliptic curve and has x and y coordinates.

(use sepc256k1)

3. __Concatenate the x and y coordinates of public key__
 then compute a Sha256 hash.

4. The resulting 32 bytes is the address !
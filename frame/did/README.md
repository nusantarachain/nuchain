# Nuchain DID

Nuchain decentralized identifiers (DIDs) pallet.

Based on [https://github.com/substrate-developer-hub/pallet-did](https://github.com/substrate-developer-hub/pallet-did).

## Overview

The DID pallet provides functionality for DIDs management. It uses a universal identity registry where all the required data is associated with an address. It enables the possibility to create a portable, persistent, privacy-protecting, and personal identity.

## Self-Sovereign Identity

A decentralized identity or self-sovereign identity is a new approach where no one but you own or control the state of your digital identity.

Some of the inherited benefits of self-sovereign identity are:

* Seamless Identity Verification
* Non-Custodial Login Solutions
* Stronger Protections for Critical Infrastructure
* Securing the Internet of Things

## Test

```
$ cargo test
```

### Dispatchable Functions

* `add_delegate` - Creates a new delegate with an expiration period and for a specific purpose.
* `change_owner` - Transfers ownership of an identity.
* `revoke_delegate` - Revokes an identity's delegate by setting its expiration to the current block number.
* `add_attribute` - Creates a new attribute as part of an identity.
* `revoke_attribute` - Revokes an attribute/property from an identity.
* `delete_attribute` - Removes an attribute from an identity. This attribute/property becomes unavailable.


License: GPL-3

[] Robin Sy.



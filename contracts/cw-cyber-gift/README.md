# cw-cyber-gift
## Claim
Claim message input:

- nickname
- avatar_cid
- claimer_addr_type
- claimer_addr
- target_addr
- relay_reward

Message example:
```json
{
    "nickname": "john",
    "avatar_cid": "QmNqdEgYJwe8QSZvcpBXqcNoG9BRoSf71q6mJLe3aRhZAW",
    "claimer_addr_type": "ethereum",
    "claimer_addr": "0xd69d55e6f8350092a75dcd9fca8f4c7b7c02f008",
    "target_addr": "bostrom1sm9kwrthaua2ywrtkwfswa7mr7cr4f5pupjgln",
    "relay_reward": "0.01"
}
```
## Generate root, proofs and verify proofs
This is a helper client shipped along contract
[merkle-airdrop-cli](https://github.com/CosmWasm/cw-tokens/tree/main/contracts/cw20-merkle-airdrop/helpers)

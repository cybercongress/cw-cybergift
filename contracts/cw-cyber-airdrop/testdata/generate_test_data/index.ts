import sha256 from 'crypto-js'
import { MerkleTree } from "merkletreejs";

class Airdrop {
  private tree: MerkleTree;

  constructor(accounts: Array<Encoding>) {
    const leaves = accounts.map((a) => this.encode_data(a));
    this.tree = new MerkleTree(leaves, sha256, { sort: true });
  }

  public getMerkleRoot(): string {
    return this.tree.getHexRoot().replace('0x', '');
  }

  public getMerkleProof(data: Encoding): string[] {
    return this.tree
      .getHexProof(this.encode_data(data))
      .map((v) => v.replace('0x', ''));
  }

  encode_data(data:Encoding): string {
    return sha256(data.target_addr + data.claim_msg + data.signature + data.amount).toString()
  }

  public verify(
    data: Encoding
  ): boolean {
    let hashBuf = Buffer.from(this.encode_data(data))

    proof.forEach((proofElem) => {
      const proofBuf = Buffer.from(proofElem, 'hex');
      if (hashBuf < proofBuf) {
        hashBuf = Buffer.from(sha256(Buffer.concat([hashBuf, proofBuf]).toString()));
      } else {
        hashBuf = Buffer.from(sha256(Buffer.concat([proofBuf, hashBuf]).toString()));
      }
    });

    return this.getMerkleRoot() === hashBuf.toString('hex');
  }
}

interface Encoding {
  target_addr: string,
  claim_msg: string,
  signature: string,
  amount: string,
}

let receivers: Array<Encoding> = JSON.parse('../airdrop_stage_1_list.json');

let airdrop = new Airdrop(receivers)

console.log(airdrop.getMerkleRoot())

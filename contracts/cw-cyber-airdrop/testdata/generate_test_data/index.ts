import CryptoJS from "crypto-js";
import sha256 from "crypto-js/sha256";
import { MerkleTree } from "merkletreejs";
import receivers from "./airdrop_stage_1_list.json";

interface Encoding {
  target_address: string;
  claim_msg: string;
  signature: string;
  amount: string;
}
class Airdrop {
  private tree: MerkleTree;

  constructor(accounts: Array<Encoding>) {
    const leaves = accounts.map((a) => this.encode_data(a));
    this.tree = new MerkleTree(leaves, sha256, { sort: true });
  }

  encode_data(data: Encoding): CryptoJS.lib.WordArray {
    return sha256(
      data.target_address + data.claim_msg + data.signature + data.amount
    );
  }

  public getMerkleRoot(): string {
    return this.tree.getRoot().toString("hex");
  }

  public getMerkleProof(data: Encoding): string[] {
    return this.tree
      .getProof(this.encode_data(data).toString())
      .map((v) => v.data.toString("hex"));
  }

  public verify(proof: [string], data: Encoding): boolean {
    return this.tree.verify(
      proof,
      this.encode_data(data).toString(),
      this.tree.getRoot()
    );
  }
}

let airdrop = new Airdrop(receivers);

console.log(airdrop.getMerkleRoot());
console.log(receivers[0])
console.log(airdrop.getMerkleProof(receivers[0]));
console.log(receivers[1])
console.log(airdrop.getMerkleProof(receivers[1]));

import {Command, flags} from '@oclif/command'
import { readFileSync, writeFileSync } from 'fs';
import CryptoJS from "crypto-js";
import sha256 from "crypto-js/sha256";
import { MerkleTree } from "merkletreejs";

interface Encoding {
  address: string;
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
      data.address + data.amount
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

class GenerateProofs extends Command {
  static description = 'Generates merkle root and proofs for given addresses'

  static examples = [
    `$ generate-merkle-proofs --input root_testing_source.json --output proof.json`,
  ]

  static flags = {
    input: flags.string({char: 'f', description: 'airdrop file location'}),
    output: flags.string({char: 'o', description: 'output file location'}),
  }

  async run() {
    const {flags} = this.parse(GenerateProofs)

    if (!flags.input) {
      this.error(new Error('Airdrop file location not defined'))
    }

    if (!flags.output) {
      this.error(new Error('Output file location not defined'))
    }

    let file = readFileSync(flags.input, 'utf-8');
    let receivers: Array<Encoding> = JSON.parse(file);
    let airdrop = new Airdrop(receivers);

    let merkle_root = airdrop.getMerkleRoot()
    console.log("Merkle root: " + merkle_root)

    let result =
      {"merkle_root": merkle_root,
       "proofs": receivers.map((r) => {return {"address": r.address, "amount": r.amount, "proof": airdrop.getMerkleProof(r)}})};
    writeFileSync(flags.output, JSON.stringify(result));
    console.log(`Number of addresses in the Merkle tree: ${Object.keys(result.proofs).length}`)
  }
}
// @ts-ignore
GenerateProofs.run().catch(require('@oclif/core/handle'))

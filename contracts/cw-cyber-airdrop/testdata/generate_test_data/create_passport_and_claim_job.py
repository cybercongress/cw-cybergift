import pandas as pd
import sys
import json
import ipfshttpclient
from contract_utils import ContractUtils
import warnings
from time import sleep

warnings.filterwarnings("ignore")

DISPLAY_TX_EXECUTION = False


def participation(row: pd.Series, address_dict: dict, release_bool: bool = False) -> dict:
    ipfs_client = ipfshttpclient.connect()
    contract_utils = ContractUtils(ipfs_client=ipfs_client, address_dict=address_dict)
    _create_passport_json = contract_utils.create_passport(row, display_data=DISPLAY_TX_EXECUTION)
    _proof_ethereum_address_json = contract_utils.proof_address(row, display_data=DISPLAY_TX_EXECUTION)
    _proof_cosmos_address_json = contract_utils.proof_address(row, network='cosmos', display_data=DISPLAY_TX_EXECUTION)
    sleep(2)
    _claim_ethereum_json = contract_utils.claim(row, display_data=DISPLAY_TX_EXECUTION)
    _claim_cosmos_json = contract_utils.claim(row, network='cosmos', display_data=DISPLAY_TX_EXECUTION)
    if release_bool:
        _release_ethereum_json = contract_utils.release(row, display_data=DISPLAY_TX_EXECUTION)
        _release_cosmos_json = contract_utils.release(row, network='cosmos', display_data=DISPLAY_TX_EXECUTION)
        return {
            'create': _create_passport_json,
            'proof_ethereum': _proof_ethereum_address_json,
            'proof_cosmos': _proof_cosmos_address_json,
            'claim_ethereum': _claim_ethereum_json,
            'claim_cosmos': _claim_cosmos_json,
            'release_ethereum': _release_ethereum_json,
            'release_cosmos': _release_cosmos_json
        }
    else:
        return {
            'create': _create_passport_json,
            'proof_ethereum': _proof_ethereum_address_json,
            'proof_cosmos': _proof_cosmos_address_json,
            'claim_ethereum': _claim_ethereum_json,
            'claim_cosmos': _claim_cosmos_json
        }


if __name__ == '__main__':

    source_file_name = sys.argv[1]
    index_number = sys.argv[2]
    gift_contract_address = sys.argv[3]

    address_dict = {gift_contract_address: 'Gift Contract',
                    'bostrom1g59m935w4kxmtfx5hhykre7w9q497ptp66asspz76vhgarss5ensdy35s8': 'Passport Contract'}
    row = pd.read_csv(source_file_name).iloc[int(index_number)]
    row['ethereum_proof'] = row['ethereum_proof'].replace('\'', '').replace('[', '').replace(']', '').split(', ')
    row['cosmos_proof'] = row['cosmos_proof'].replace('\'', '').replace('[', '').replace(']', '').split(', ')

    res = participation(row, address_dict)

    with open(f'temp/contract_execution_log_{index_number}.txt', 'w') as convert_file:
        convert_file.write(json.dumps(res))
    print(f"{row['bostrom_address']}: done")

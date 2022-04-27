import pandas as pd
import sys
import json
from contract_utils import ContractUtils
import warnings
from time import sleep
from aiohttp.client_exceptions import ClientConnectorError
from cyber_sdk.exceptions import LCDResponseError

warnings.filterwarnings("ignore")

DISPLAY_TX_EXECUTION = False


def participation(row: pd.Series, address_dict: dict, release_bool: bool = False) -> dict:

    contract_utils = ContractUtils(ipfs_client=None, address_dict=address_dict)
    if release_bool:
        _release_ethereum_json = contract_utils.release(row, display_data=DISPLAY_TX_EXECUTION)
        sleep(1)
        _release_cosmos_json = contract_utils.release(row, network='cosmos', display_data=DISPLAY_TX_EXECUTION)
        return {
            'release_ethereum': _release_ethereum_json,
            'release_cosmos': _release_cosmos_json
        }
    else:
        _create_passport_json = contract_utils.create_passport(row, display_data=DISPLAY_TX_EXECUTION)
        sleep(1)
        _proof_ethereum_address_json = contract_utils.proof_address(row, display_data=DISPLAY_TX_EXECUTION)
        sleep(1)
        _proof_cosmos_address_json = contract_utils.proof_address(row, network='cosmos',
                                                                  display_data=DISPLAY_TX_EXECUTION)
        sleep(1)
        _claim_ethereum_json = contract_utils.claim(row, display_data=DISPLAY_TX_EXECUTION)
        sleep(1)
        _claim_cosmos_json = contract_utils.claim(row, network='cosmos', display_data=DISPLAY_TX_EXECUTION)
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
    release_bool = True if len(sys.argv) > 4 and sys.argv[4] == 'True' else False

    log_file = f'temp/contract_release_execution_log_{index_number}.txt' \
        if release_bool else f'temp/contract_participation_execution_log_{index_number}.txt'
    address_dict = {gift_contract_address: 'Gift Contract',
                    'bostrom1g59m935w4kxmtfx5hhykre7w9q497ptp66asspz76vhgarss5ensdy35s8': 'Passport Contract'}

    row = pd.read_csv(source_file_name).iloc[int(index_number) % 10_000]
    row['ethereum_proof'] = row['ethereum_proof'].replace('\'', '').replace('[', '').replace(']', '').split(', ')
    row['cosmos_proof'] = row['cosmos_proof'].replace('\'', '').replace('[', '').replace(']', '').split(', ')

    res = None
    last_error = None
    i = 0
    while res is None and i < 5:
        try:
            res = participation(row=row, address_dict=address_dict, release_bool=release_bool)
        except (ClientConnectorError, LCDResponseError) as e:
            sleep(10)
            print(f'Error: {e}\n')
            last_error = e
            i += 1
    if res is not None:
        with open(log_file, 'w') as convert_file:
            convert_file.write(json.dumps(res))
        print(f"{row['bostrom_address']}: done")
    elif last_error is not None:
        with open(log_file, 'w') as convert_file:
            convert_file.write(f'Error: {last_error}')
        print(f"{row['bostrom_address']}: unsuccessful")

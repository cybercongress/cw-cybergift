import json
from subprocess import Popen, PIPE

NODE_URL = 'https://rpc.space-pussy-1.cybernode.ai:443'
NETWORK = 'space-pussy-1'


def execute_bash(bash_command: str, display_data: bool = False) -> [str, str]:
  _output, _error = Popen(bash_command, shell=True, stdout=PIPE, stderr=PIPE).communicate()
  if _error:
    display_data = True
  if display_data:
    print(bash_command)
    if _output:
      try:
        print(json.dumps(json.loads(_output.decode('utf-8')), indent=4, sort_keys=True))
      except json.JSONDecodeError:
        print(_output)
    if _error:
      print(_error)
  return _output.decode('utf-8'), _error.decode('utf-8')


def instantiate_contract(init_query: str, contract_code_id: str, contract_label: str, amount: str = '',
                           display_data: bool = False) -> str:
    _init_output, _init_error = execute_bash(
      f'''INIT='{init_query}' \
            && cyber tx wasm instantiate {contract_code_id} "$INIT" --from $WALLET {'--amount ' + amount + 'boot' if amount else ''} --label "{contract_label}" \
            -y --gas 3500000 --broadcast-mode block -o json --chain-id={NETWORK} --node={NODE_URL}''')
    if display_data:
      try:
        print(json.dumps(json.loads(_init_output.decode('utf-8')), indent=4, sort_keys=True))
      except json.JSONDecodeError:
        print(_init_output)
    if _init_error:
      print(_init_error)
    _init_json = json.loads(_init_output)
    return \
    [event['attributes'][0]['value'] for event in _init_json['logs'][0]['events'] if event['type'] == 'instantiate'][0]


def execute_contract(execute_query: str, contract_address: str, from_address: str = '$WALLET', gas: int = 300000,
                     display_data: bool = False) -> str:
  _execute_output, _execute_error = execute_bash(
    f'''EXECUTE='{execute_query}' \
            && CONTRACT="{contract_address}" \
            && cyber tx wasm execute $CONTRACT "$EXECUTE" --from {from_address} --broadcast-mode block -o json -y --gas={gas} --chain-id={NETWORK} --node={NODE_URL}''')
  if display_data:
    try:
      print(json.dumps(json.loads(_execute_output), indent=4, sort_keys=True))
    except json.JSONDecodeError:
      print(_execute_output)
  if _execute_error:
    print(_execute_error)
  return _execute_output


def query_contract(query: str, contract_address: str, display_data: bool = False) -> json:
  _execute_output, _execute_error = execute_bash(
    f'''QUERY='{query}' \
            && cyber query wasm contract-state smart {contract_address} "$QUERY" -o json --chain-id={NETWORK} --node={NODE_URL}''')
  try:
    if display_data:
      print(json.dumps(json.loads(_execute_output), indent=4, sort_keys=True))
    return json.loads(_execute_output)
  except json.JSONDecodeError:
    print(_execute_output)
  if _execute_error:
    print(_execute_error)
  return json.loads(_execute_output)

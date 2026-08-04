# Demo test suite

Python tests for the dispute-lifecycle demo layer. These complement the Rust
integration tests (`framework/tests/framework_tests.rs`) and the Foundry tests
(`framework/test/DisputeRegistry.t.sol`).

## Layout

| File | What it covers | Needs Rust? |
| --- | --- | --- |
| `test_dispute_schema.py` | JSON-Schema validation of `dispute_record` / `dispute_resolution` payloads and their envelope binding (valid + invalid enum / pattern / required / status cases). | No |
| `test_dispute_flow.py` | End-to-end offline terms-mismatch + refund flow via `run_dispute_flow.py`: artifact shape, settlement-state trail, event trail, and envelope schema validity. | Yes (`cargo`) — auto-skipped if absent |

## Install test deps

```bash
python3 -m pip install -r demo/tests/requirements.txt
```

## Run

From the repository root (`covenant-layer/`):

```bash
python3 -m unittest discover -s demo/tests -v
```

Run a single module:

```bash
python3 -m unittest demo.tests.test_dispute_schema -v
```

The flow test shells out to `cargo run` in local mode; it is skipped
automatically when the Rust toolchain is unavailable.

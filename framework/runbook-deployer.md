# Registry Deployer Runbook

Use this runbook to deploy the reference `ProviderRegistry` contract and connect framework clients.

## Ownership model

- Contract owner/governance should be a multisig, not an EOA.
- Deployment key should only bootstrap and transfer governance.
- Governance address must be published in public docs and `networks.yaml`.

## Deployment steps (example with Foundry)

1. Install Foundry and configure `RPC_URL` + `PRIVATE_KEY`.
   - also set `GOVERNANCE_ADDRESS` (multisig/DAO executor).
2. Compile contract:

```bash
forge build
```

3. Deploy with the provided helper script:

```bash
cp .env.example .env
source .env
./scripts/deploy_registry.sh
```

Or deploy manually:

```bash
forge create contracts/ProviderRegistry.sol:ProviderRegistry \
  --rpc-url "$RPC_URL" \
  --private-key "$PRIVATE_KEY" \
  --constructor-args "$GOVERNANCE_ADDRESS"
```

4. Record deployed address in `framework/networks.yaml` under target network.

5. Verify deployment ownership:

- `governance()` returns multisig.
- Governance can transition participant state.
- Non-governance cannot transition participant state.

## Post-deploy checks

- Test `registerParticipant` from independent wallet.
- Test `transitionParticipant` from governance only.
- Confirm events are emitted and indexable.
- Publish contract address + chain id in public governance doc.

## Incident response

- If governance key compromise is suspected:
  - rotate governance via `setGovernance` from current governance
  - publish incident statement and migration timeline
  - optionally pause operator routing to non-active participants until resolved

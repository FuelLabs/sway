# Deploy and call a Sway contract with Typescript

This guide walks through the steps for deploying and calling a Sway contract in Typescript. Go [here](https://github.com/FuelLabs/fuels-ts) for full documentation on the Typescript SDK.

## 7. Deploy `wallet_contract` with Typescript SDK

When you built `wallet_contract` using `forc build`, this should have created a `wallet_contract.bin` file in the `out/debug` subdirectory and a `wallet_contract-abi.json` file in the `out/debug` subdirectory. Check and make sure you have it because you'll need it below.

In your Typescript application, copy and paste the following code to set up a local node, compile and deploy the `wallet_contract`.

```typescript
import { Provider, Contract } from "fuels";
import

const provider = new Provider("http://127.0.0.1:4000/graphql");

// Deploy
const bytecode = fs.readFileSync(path.join(__dirname, "./wallet_contract.bin"));
const salt = genSalt();
const { contractId } = await provider.submitContract(bytecode, salt);
const contract = MyContract__factory.connect(contractId, provider);

const req = SendFundsRequest {
    amount_to_send: 200,
    recipient_address: 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b,
};

// Call
const result = await contract.functions.send_funds(454, 232, ETH_ID, req)

// Assert
expect(result.toNumber()).toEqual(1337);
```

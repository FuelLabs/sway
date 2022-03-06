# Deploy and Call a Sway Contract With TypeScript

This guide walks through the steps for deploying and calling a Sway contract in TypeScript. Go [here](https://github.com/FuelLabs/fuels-ts) for full documentation on the TypeScript SDK.

## Deploy `wallet_contract` With the TypeScript SDK

When you built `wallet_contract` using `forc build`, this should have created `wallet_contract.bin` and `wallet_contract-abi.json` in `out/debug`. Check and make sure you have it because you'll need it below.

In your TypeScript application, copy and paste the following code to set up a local node, compile and deploy the `wallet_contract`.

```typescript
import { Provider, Contract } from "fuels";

const provider = new Provider("http://127.0.0.1:4000/graphql");

// Deploy
const bytecode = fs.readFileSync(path.join(__dirname, "./wallet_contract.bin"));
const salt = genSalt();
const { contractId } = await provider.submitContract(bytecode, salt);
const contract = MyContract__factory.connect(contractId, provider);

// Call
const result = await contract.functions.send_funds(
  200,
  0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b,
);

// Assert
expect(result.toNumber()).toEqual(1337);
```

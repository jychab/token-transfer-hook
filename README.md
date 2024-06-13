# Motivation

The purpose of this smart contract is to create a token with a unique fee mechanism designed to encourage wide distribution of the token. By incentivizing senders to transfer tokens to new wallets, we aim to foster greater adoption and circulation of the token. This is achieved through a referral system that rewards users for bringing new participants into the token ecosystem.

## How It Works
Initial Transfer and Referral Assignment:

- When a user (the sender) transfers X tokens to another wallet (the receiver), the smart contract checks if the receiver already holds any X tokens.
- If the receiver does not hold any X tokens, the smart contract assigns the sender as the referrer for the receiver.
  
Earning Referral Fees:

- Once a receiver has an assigned referrer, any subsequent transfers of X tokens made by the receiver will include a referral fee.
- A percentage of the X tokens from each transfer made by the receiver will be automatically sent to their assigned referrer.
  
Resetting Referrer Assignment:

- If a wallet holds no X tokens at any point, the referrer assignment for that wallet will be reset to null.
- This means that if the receiver (who has a referrer assigned) transfers out all their X tokens and their balance reaches zero, the referral relationship is terminated.

## Example Scenario

Alice Sends Tokens to Bob:

- Alice transfers X tokens to Bob's wallet.
- The smart contract checks Bob’s wallet and sees that he has no X tokens.
- Alice is now assigned as Bob’s referrer.

Bob Transfers Tokens to Charlie:

- Bob later transfers X tokens to Charlie.
- A small percentage of X tokens from Bob’s transfer is automatically sent to Alice as a referral fee because Alice is Bob's assigned referrer.
- If Charlie does not hold any X tokens, Bob is now assigned as Charlie’s referrer.

Bob’s Wallet Balance Hits Zero:

- If Bob’s balance of X tokens drops to zero at any point, his referrer assignment (Alice) is reset.
- If Bob receives X tokens again in the future, the referral assignment process will start anew.

## Benefits
- Encourages Distribution: Users are motivated to transfer tokens to new wallets to earn referral fees.
- Promotes Engagement: As users benefit from the growth of the network, they are incentivized to encourage others to join and use the token.
- Dynamic Referrals: The referral relationships are dynamic and adapt to the token holding patterns of users, ensuring active participants are rewarded.

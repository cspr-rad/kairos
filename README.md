# Kairos: Native Cspr rollup circuit

Types needed:

    - a leaf
    - previous state
    - updated state

    I will only update the accounts of those users that are affected by a state mutation
    -> Set of transfers in, updated balances + proof out

    -> The L1 contract will resolve the output struct that is a mapping of {
        Account: Balance
    }

    The L2 stores all Balances and uses lookup whenever an account is affected by a transaction

    Deposits are executed first, then Transfers
    -> update state in side the risc0 circuit, looping over all transfers
    -> Transfers must be valid for the current balance (L2)
    -> Transfers immediately decrease the L2 balance, 

    L1 Withdrawals are not possible => all withdrawals have to be routed through the L2


# Kairos deposit contract
A deposit contract for the Kairos L2, targeting Casper Node v2.0 and VM v1.0

# Entry Points
Documentation of all contract entry points

## Update Admin - ADMIN
Updates the admin account. Admin account is permitted to re-create the contract purse and to execute withdrawals.

Arguments:

- new_admin: Key

Return:

()

## Create Purse - ADMIN
Create a new contract purse => overrides the value under the corresponding contract purse named_key

Arguments: 

None

Return:

()

## Get Purse
Retrieve a reference to the deposit purse with "ADD" permission

Arguments:

None

Return:

URef

## Deposit
Execute a deposit on the L1.

Arguments:

- temp_purse: URef
- amount: U512

Return:

()

## Withdrawal - ADMIN
Withdrawal from the contract purse, can only be called by the admin.

Arguments:

- destination_purse: URef
- amount: U512

Return:

()

## Increment most recent Deposit Counter - ADMIN
This entry point is called by the L2 whenever a deposit request is routed through the L2 (not created through a native L1 call). This method updates the counter of the total deposits submitted through both L1 and L2.

Arguments:

None

Return: 

()

# User 
    
    -> main_purse

    -> temp_purse

1. Call the session code to transfer N tokens from main_purse to the freshly created temp_purse.

2. The session code will then call the deposit entry point of the deposit contract and pass the temp_purse as a reference.

3. All funds from the temp_purse will be transferred to the contract_purse that is managed by the ADMIN.


## Building and testing
Currently the paths in build-macos-darwin.sh are configured for Macos, nixifying this makes sense. Paths must be discoverable by utils.rs in the test engine.

For now, on Macos:

`./build-macos-darwin-feat-2.0.sh`

or, on Ubuntu/Debian:

`./build-ubuntu-feat-2.0.sh`

then:

```bash
cd tests
cargo test
```

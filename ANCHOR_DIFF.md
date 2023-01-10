# Differences with ANCHOR contracts (work in progress)

In this file, you will find all the differences between this repository and the original ANCHOR repository, located at : https://github.com/Anchor-Protocol/money-market-contracts


## Custody Base
	Major changes : 
		None
	Some changes in syntax.
	Relaxed some tests

## Custody BLuna
	Major Changes : 
		1. The price querier and native swaps don't exist anymore
			We had to switch to price queries and swaps using Terra dexes. 
			We chose 3 sources of truth for the price of the asset --> Astroport, PhoenixSwap and TerraSwap
	Some changes in tests and syntax.
	Removed the tax, as there is no more native assets tax on Terra
	Relaxed some tests

## Distribution Model
	Major Changes : 
		None
	Modified variable names. No more anc emission, borrower incentives are used instead.

## Interest Model
	Major Changes : 
		None
	Adapted to the new Decimal256 structure from changes between cosmwasm@0.16 and cosmwasm@1.1.

## Liquidation Queue
	Major Changes : 
		None
	Adapted to the new Decimal256 structure from changes between cosmwasm@0.16 and cosmwasm@1.1.
	Some changes in syntax.
	Removed the tax system
	Somes changes in tests.

## Market Contract
	Major Changes : 
		1. Removed the ANC reward system	
		2. Added the borrower inceitives mechanism
			This change is very sensitive and should be looked at first when searching for contract failures
			get_actual_interest_factor
			Especially to make sure that subsequent queries and execute messages (especially deposit, repay...) don't interset with execute_epoch_updates.
	Adapted to the new Decimal256 structure from changes between cosmwasm@0.16 and cosmwasm@1.1.
	Removed the tax system
	Somes changes in tests.

## Oracle
	Major Changes : 
		None
	Adapted to the new Decimal256 structure from changes between cosmwasm@0.16 and cosmwasm@1.1.

## Overseer
	Major Changes : 
		1. Removed the ANC reward system	
		2. Added the borrower incentives mechanism
			This change is very sensitive and should be looked at first when searching for contract failures
			get_actual_interest_factor
			Especially to make sure that subsequent queries and execute messages (especially deposit, repay...) don't interset with execute_epoch_updates.
	Adapted to the new Decimal256 structure from changes between cosmwasm@0.16 and cosmwasm@1.1.
	Removed the tax system
	Somes changes in tests.
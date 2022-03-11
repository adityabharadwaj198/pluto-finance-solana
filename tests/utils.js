const {createMint, u64} = require('@solana/spl-token')
const TokenInstructions = require('@project-serum/serum').TokenInstructions
const anchor = require('@project-serum/anchor')

const {SystemProgram} = anchor.web3;

const createPriceFeed = async ({
  admin,
  oracleProgram,
  initPrice,
  ticker
}) => {
  const collateralTokenPriceFeed = anchor.web3.Keypair.generate(); //namespace -> web3, class -> Account (An account key pair (public and secret keys).)
  //TODO: change this to account if It gives error
  console.log("see the publickeys", collateralTokenPriceFeed.publicKey.toString())
  console.log("see the parameters passed", admin, oracleProgram, initPrice, ticker);
  let tx = await oracleProgram.rpc.initializeOracle(admin.publicKey, initPrice, ticker, {
    accounts: {
      priceFeed: collateralTokenPriceFeed.publicKey, //account was generated on L#12 -> this will store all your price related changes. 
      admin: admin.publicKey,
      systemProgram: SystemProgram.programId,
    },
    signers: [collateralTokenPriceFeed],
  })

  console.log("yo see this transaction inside initializePriceFeed", tx);
  return collateralTokenPriceFeed
}

const createToken = async({provider, mintAuthority}) => {
  const token = await createMint(
        provider.connection,
        provider.wallet.payer,
        mintAuthority,
        null,
        8,
    );
    console.log("printing token", token);
    return token;
}

const tou64 = (amount) => {
  // eslint-disable-next-line new-cap
  return u64(amount.toString())
}

const createAccountWithCollateral = async ({
    systemProgram,
    mintAuthority,
    collateralToken,
    collateralAccount,
    amount = new anchor.BN(100 * 1e8)  
}) => {
    const userWallet = await newAccountWithLamports(systemProgram.provider.connection)
    const userAccount = new anchor.web3.Account()
    await systemProgram.rpc.createUserAccount(userWallet.publicKey, {
      accounts: {
        userAccount: userAccount.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY
      },
      signers: [userAccount],
      // Auto allocate memory
      instructions: [await systemProgram.account.userAccount.createInstruction(userAccount)] 
    })
    const userCollateralTokenAccount = await collateralToken.createAccount(userWallet.publicKey) //user's collateral token account is being created
    await collateralToken.mintTo( //collateral jo rakhna hai token that will appear in user's collateral token account
      userCollateralTokenAccount,
      mintAuthority,
      [],
      tou64(amount.toString())
    ) 
}

const newAccountWithLamports = async (connection, lamports = 1e10) => {
  const account = new anchor.web3.Account()

  let retries = 30
  await connection.requestAirdrop(account.publicKey, lamports)
  for (;;) {
    await sleep(500)
    // eslint-disable-next-line eqeqeq
    if (lamports == (await connection.getBalance(account.publicKey))) {
      return account
    }
    if (--retries <= 0) {
      break
    }
  }
  throw new Error(`Airdrop of ${lamports} failed`)
}

const sleep = (ms) => {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

module.exports = {
  createToken,
  createAccountWithCollateral,
  createPriceFeed,
  // mintUsd,
  // updateAllFeeds,
  tou64,
  newAccountWithLamports
}

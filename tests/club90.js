const anchor = require('@project-serum/anchor');
const { TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount, getAccount, createAccount, mintTo, transfer, createTransferInstruction} = require('@solana/spl-token');
const { min } = require('bn.js');
const {  createToken, newAccountWithLamports, createPriceFeed, tou64} = require('./utils.js')
var assert = require('assert');
const {SystemProgram} = anchor.web3;

describe('club90', () => {
  console.log("Starting test")
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);
  const wallet = provider.wallet.payer;
  const connection = provider.connection;
  const admin = wallet;
  const club90Program = anchor.workspace.Club90;
  const oracleProgram = anchor.workspace.Oracle;
  const baseAccount = anchor.web3.Keypair.generate(); 
  const signer = provider.wallet.publicKey;
  let collateralToken
  let mintAuthority
  let collateralAccount
  let syntheticUSD
  let nonce 
  let collateralTokenFeed
  const initPrice = new anchor.BN(2 * 1e4)
  const ticker = Buffer.from('PLTO', 'utf-8')
  const debt = 0;
  const shares = 0;
  const collateral_balance = 0;
  const collateralization_level = 500;
  const max_delay = 1000;
  const fee = 30;

  before (async() => { 

    console.log('Starting stuff off with StartStuffOff!')
    const tx = await club90Program.rpc.startStuffOff({ //pre-initializing (starting stuff off) baseAccount and all the variables present inside it 
      accounts: {
        baseAccount: baseAccount.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [baseAccount],
    })
    console.log("Your transaction signature for calling StartStuffOff", tx);

   let collateralTokenPriceFeed = await createPriceFeed({admin, oracleProgram, initPrice, ticker})
    console.log("Printing collateralTokenPriceFeed", collateralTokenPriceFeed);

    const [_mintAuthority, _nonce] = await anchor.web3.PublicKey.findProgramAddress( //_ means private //Find Program address given a PublicKey
      // [Buffer.from("aditya"), signer.toBuffer()],
      [signer.toBuffer()],
      club90Program.programId //this is publicKey
    )
    nonce = _nonce
    mintAuthority = _mintAuthority
    console.log("printing nonce and mintauthority", nonce, mintAuthority)
    collateralToken = await createToken({provider, mintAuthority: wallet.publicKey }) //collateral token that the user will keep as collateral (PLTO)
    collateralAccount = await getOrCreateAssociatedTokenAccount(connection, provider.wallet.payer,
       collateralToken, provider.wallet.payer.publicKey);
    console.log("collateralAccount is --->", collateralAccount);
    syntheticUSD = await createToken({provider, mintAuthority})

    console.log("Printing collateralToken, collateralAccount, syntheticUSD", collateralToken, collateralAccount, syntheticUSD);

    console.log('Initializing baseAccount variables to default values using initialize!');

    const tx3 = await club90Program.rpc.initialize(nonce, signer, admin.publicKey,
                  collateralToken, collateralAccount.address, collateralTokenPriceFeed.publicKey,  //look for example here https://spl.solana.com/token to understand why I'm passing .address.publicKey
                  syntheticUSD, mintAuthority.publicKey, 
                  {
                    accounts: {
                    baseAccount: baseAccount.publicKey,
                  }
                });

    console.log('Transaction for initialize was', tx3);            
  })
  it('Checking if initialize worked OK!', async () => {
    // Add your test here.
    let account = await club90Program.account.baseAccount.fetch(baseAccount.publicKey);
    assert.ok(account.nonce === nonce);
    assert.ok(account.initialized === true);
    // console.log("1---- account.signer ----", account.signer, signer);
    assert.ok(account.signer.equals(signer));
    // console.log("2---- account.collateralToken ----", account.collateralToken, collateralToken);
    assert.ok(account.collateralToken.equals(collateralToken));
    // console.log("3---- account.collateralAccount ----", account.collateralAccount, collateralAccount.address);
    assert.ok(account.collateralAccount.equals(collateralAccount.address));
    // console.log("4---- account.debt ----", account.debt, debt);
    assert.ok(account.debt.eq(new anchor.BN(0)));
    // console.log("5---- account.shares ----", account.shares, shares);
    assert.ok(account.shares.eq(new anchor.BN(0)));
    // initaly we will have collateral and sythetic usd
    // console.log("6---- account.assets----", account.assets);
    assert.ok(account.assets.length === 2);
    // console.log("7---- synt asset price ----", account.assets[0].price);
    assert.ok(account.assets[0].price.eq(new anchor.BN(1e4)));
    // console.log("8 ---- synth asset address", account.assets[0].assetAddress);
    assert.ok(account.assets[0].assetAddress.equals(syntheticUSD));
    // initial collateralBalance
    const collateralAccountInfo = await getAccount(connection, collateralAccount.address);
    // console.log("Printing collateralAccountInfo", collateralAccountInfo.amount); //check here for using getAccount(https://spl.solana.com/token)
    assert.ok(collateralAccountInfo.amount == (new anchor.BN(0))); 
  });
  
  it('Checking deposit function ', async () => {
    const userWallet = await newAccountWithLamports(club90Program.provider.connection) //custom function creates acc using account = new anchor.web3.Account() & gets sol airdropped.
    console.log("Got user wallet", userWallet.publicKey.toString());
    const userAccount = anchor.web3.Keypair.generate();
    console.log("see the userAccount", userAccount.publicKey.toString());
    const tx5 = await club90Program.rpc.createUserAccount(userWallet.publicKey, { //userWallet.publicKey will be the owner. 
      accounts: {
        userAccount: userAccount.publicKey, //this account is for storing user data on chain
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId, //???
      },
      signers: [userAccount],
    })
    console.log("see the createUserAccount txn hash", tx5);
    const account = await club90Program.account.userAccount.fetch(userAccount.publicKey) //this is for fetching userAccount
    console.log("Created user's account on chain", account);
    assert.ok(account.shares.eq(new anchor.BN(0))) //checking if userAccount initialization was OK 
    assert.ok(account.collateral.eq(new anchor.BN(0))) //same ^^^
    assert.ok(account.owner.equals(userWallet.publicKey)) /// ^^^
    const userCollateralTokenAccount = await getOrCreateAssociatedTokenAccount(connection, provider.wallet.payer,
      collateralToken, userWallet.publicKey); //collateralToken's acc for the user. 
    console.log("created user's collateral token account", userCollateralTokenAccount);
    const amount = new anchor.BN(10) 
    const tx6 = await mintTo(provider.connection, provider.wallet.payer, collateralToken, userCollateralTokenAccount.address, provider.wallet.payer, amount); //check: https://solana-labs.github.io/solana-program-library/token/js/modules.html#mintTo
    console.log("see tx6 hash", tx6);
    const userCollateralTokenAccountInfo = await getAccount(provider.connection, userCollateralTokenAccount.address);
    console.log("see userCollateralTokenAccountInfo", userCollateralTokenAccountInfo);
    assert.ok(userCollateralTokenAccountInfo.amount == (new anchor.BN(amount)))
    console.log("printing system's collateralAccount", collateralAccount);
    const tx7 = await club90Program.rpc.deposit({ //https://project-serum.github.io/anchor/ts/index.html#RpcNamespace
      accounts: {
        baseAccount: baseAccount.publicKey,
        userAccount: userAccount.publicKey, //user's account defined on L91
        collateralAccount: collateralAccount.address, //system's collateral account
      },
      signers: [userWallet],
      instructions: [ //https://solana-labs.github.io/solana-program-library/token/js/modules.html#createTransferInstruction
        createTransferInstruction(
          userCollateralTokenAccount.address,
          collateralAccount.address, 
          userWallet.publicKey, 
          BigInt(5), 
          collateralToken.programId,
          )
       ]
    });
    console.log("transaction hash tx7", tx7);
    const systemCollateralAccountInfo = await getAccount(provider.connection, collateralAccount.address);
    console.log("see the system collateral account must now contain 5", systemCollateralAccountInfo);
    assert.ok(systemCollateralAccountInfo.amount == (new anchor.BN(5)));
    const accountAfterDeposit = await club90Program.account.userAccount.fetch(userAccount.publicKey)
    console.log("see the account after depositing", accountAfterDeposit);
    assert.ok(accountAfterDeposit.shares.eq(new anchor.BN(0)))
    assert.ok(accountAfterDeposit.collateral.eq(new anchor.BN(5)))
    assert.ok(accountAfterDeposit.owner.equals(userWallet.publicKey))
  })
});

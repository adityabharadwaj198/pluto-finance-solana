const anchor = require('@project-serum/anchor');
const { getOrCreateAssociatedTokenAccount, getAccount } = require('@solana/spl-token');
const { min } = require('bn.js');
const {  createToken, createAccountWithCollateral, createPriceFeed} = require('./utils.js')
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

  //before each -> runs before each test in this block 
  //after -> runs once after the last test in this block 
  // after each -> runs after earch test in this block
  before (async() => { //runs once before the first test in this block 

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

    /*
    Below will return a program address and a nonce. (Q: why do I need a program address for mintAuthority?)
    */
    const [_mintAuthority, _nonce] = await anchor.web3.PublicKey.findProgramAddress( //_ means private //Find Program address given a PublicKey
      // [Buffer.from("aditya"), signer.toBuffer()],
      [signer.toBuffer()],
      club90Program.programId //this is publicKey
    )
    nonce = _nonce
    mintAuthority = _mintAuthority
    console.log("printing nonce and mintauthority", nonce, mintAuthority)
    collateralToken = await createToken({provider, mintAuthority: wallet.publicKey }) //collateral token banaya (feed use nahi kiya?)
    collateralAccount = await getOrCreateAssociatedTokenAccount(connection, provider.wallet.payer, collateralToken, provider.wallet.payer.publicKey);
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
    console.log("1---- account.signer ----", account.signer, signer);
    assert.ok(account.signer.equals(signer));
    console.log("2---- account.collateralToken ----", account.collateralToken, collateralToken);
    assert.ok(account.collateralToken.equals(collateralToken));
    console.log("3---- account.collateralAccount ----", account.collateralAccount, collateralAccount.address);
    assert.ok(account.collateralAccount.equals(collateralAccount.address));
    console.log("4---- account.debt ----", account.debt, debt);
    assert.ok(account.debt.eq(new anchor.BN(0)));
    console.log("5---- account.shares ----", account.shares, shares);
    assert.ok(account.shares.eq(new anchor.BN(0)));
    // initaly we will have collateral and sythetic usd
    console.log("6---- account.assets----", account.assets);
    assert.ok(account.assets.length === 2);
    console.log("7---- synt asset price ----", account.assets[0].price);
    assert.ok(account.assets[0].price.eq(new anchor.BN(1e4)));
    console.log("8 ---- synth asset address", account.assets[0].assetAddress);
    assert.ok(account.assets[0].assetAddress.equals(syntheticUSD));
    // initial collateralBalance
    const collateralAccountInfo = await getAccount(connection, collateralAccount.address);
    console.log("Printing collateralAccountInfo", collateralAccountInfo.amount); //check here for using getAccount(https://spl.solana.com/token)
    assert.ok(collateralAccountInfo.amount == (new anchor.BN(0))); 
  });
  
});

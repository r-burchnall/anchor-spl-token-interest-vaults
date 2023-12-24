import * as anchor from "@coral-xyz/anchor";
import {Program, Wallet} from "@coral-xyz/anchor";
import {GfxTokenVaults} from "../target/types/gfx_token_vaults";
import {
    Connection,
    Keypair,
    LAMPORTS_PER_SOL,
    PublicKey,
    sendAndConfirmTransaction,
    SystemProgram, Transaction
} from "@solana/web3.js";
import {expect} from "chai";
import {
    createMint,
    getAssociatedTokenAddress,
    getMint, getOrCreateAssociatedTokenAccount,
    Mint, mintTo,
    TOKEN_PROGRAM_ID
} from "@solana/spl-token";

function createWallet(): { keypair: Keypair, wallet: Wallet } {
    const keypair = Keypair.generate();
    const wallet = new Wallet(keypair);

    return {keypair, wallet}
}

async function airdropSol(connection: Connection, wallet: PublicKey, amount = 1) {
    const balance = await connection.getBalance(wallet)
    if (balance < LAMPORTS_PER_SOL * amount) {
        const tx = await connection.requestAirdrop(wallet, LAMPORTS_PER_SOL * amount)
        await connection.confirmTransaction(tx);
    }
}

async function topUpMint(connection:Connection,mint:Mint,mintSigner:Keypair,wallet:Wallet,amount=10){
    const tokenAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        wallet.payer,
        mint.address,
        wallet.publicKey
    )
   const tx= await mintTo(connection,wallet.payer,mint.address,tokenAccount.address,mintSigner,amount*LAMPORTS_PER_SOL)
    console.log(`Topped up ${amount} txn: ${tx}`)
}
describe("gfx-token-vaults", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
    const {connection} = anchor.getProvider()

    // Generate a new keypair and wallet for each test run
    const {keypair: userKeyPair, wallet: userWallet} = createWallet()

    // If you want to reuse your keypair, you can use the following line instead
    // const keydata: number[] = require("path/to/id.json")
    // const userKeyPair = Keypair.fromSeed(Uint8Array.from(keydata.slice(0, 32)))
    // const userWallet = new Wallet(userKeyPair)

    const program = anchor.workspace.GfxTokenVaults as Program<GfxTokenVaults>;

    let mint: Mint;
    let mintAuthorityWallet: Wallet;

    before(async () => {
        await airdropSol(connection, userKeyPair.publicKey, 10)

        const {keypair: mintAuthority, wallet} = createWallet()
        mintAuthorityWallet = wallet
        await airdropSol(connection, mintAuthority.publicKey, 10)

        const mintAddress = await createMint(
            connection,
            mintAuthority, // payer
            mintAuthority.publicKey, // mint authority
            null,
            9 // We are using 9 to match the CLI decimal default exactly
        );
        mint = await getMint(connection, mintAddress)
        await topUpMint(connection,mint,mintAuthority,userWallet)
    })

    it("Is initialized!", async () => {
        const [vaultKey] = PublicKey.findProgramAddressSync([Buffer.from("vault","utf-8"),mint.address.toBuffer(), userKeyPair.publicKey.toBuffer()], program.programId)
        const [vaultOffset] = PublicKey.findProgramAddressSync([mint.address.toBuffer(),vaultKey.toBuffer()], program.programId)

        const inst = await program.methods.initialize()
            .accounts({
                owner: userKeyPair.publicKey,
                vault: vaultKey,
                mint: mint.address,
                newAta: vaultOffset,

                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .instruction()

        const tx = new Transaction();
        tx.add(inst)
        const hash = await sendAndConfirmTransaction(connection, tx, [userWallet.payer], {skipPreflight: true})
        console.log(`Your transaction signature https://explorer.solana.com/tx/${hash}?cluster=custom`);
    });

    it("should allow me to deposit", async () => {
        const [vaultKey] = PublicKey.findProgramAddressSync([Buffer.from("vault","utf-8"), mint.address.toBuffer(),userKeyPair.publicKey.toBuffer()], program.programId)

        const interestVault = await program.account.interestVault.fetch(vaultKey)

        const inst = await program.methods.deposit(new anchor.BN(1))
            .accounts({
                signer: userKeyPair.publicKey,
                vault: vaultKey,
                toAta: interestVault.ataAddress,
                fromAta: await getAssociatedTokenAddress(mint.address, userWallet.publicKey),

                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([userKeyPair])
            .instruction()

        const tx = new Transaction();
        tx.add(inst)

        const hash = await sendAndConfirmTransaction(connection, tx, [userKeyPair], {skipPreflight: true})
        console.log("Your transaction signature", hash);

        const bal = await program.account.interestVault.fetch(vaultKey)
        expect(bal.balance.toNumber()).to.eq(1)
    })

    it("should allow me to withdraw", async () => {
        const [vaultKey] = PublicKey.findProgramAddressSync([Buffer.from("vault","utf-8"), mint.address.toBuffer(),userKeyPair.publicKey.toBuffer()], program.programId)

        const interestVault = await program.account.interestVault.fetch(vaultKey)

        const ix = await program.methods.withdraw(new anchor.BN(1))
            .accounts({
                signer: userKeyPair.publicKey,
                vault: vaultKey,
                toAta: await getAssociatedTokenAddress(mint.address, userWallet.publicKey),
                fromAta: interestVault.ataAddress,

                tokenProgram: TOKEN_PROGRAM_ID,
            })

            .signers([userKeyPair])
            .instruction();

        const tx = new Transaction();
        tx.add(ix)

        const hash = await sendAndConfirmTransaction(connection, tx, [userKeyPair], {skipPreflight: true})
        console.log("Your transaction signature", hash);

        const bal = await program.account.interestVault.fetch(vaultKey)
        expect(bal.balance.toNumber()).to.eq(0)
    })

    it("should allow me to calculate interest", async () => {
        const [vaultKey] = PublicKey.findProgramAddressSync([Buffer.from("vault","utf-8"), mint.address.toBuffer(),userKeyPair.publicKey.toBuffer()], program.programId)

        const interestVault = await program.account.interestVault.fetch(vaultKey)

        let ix = await program.methods.deposit(new anchor.BN(100000))
            .accounts({
                signer: userKeyPair.publicKey,
                vault: vaultKey,
                toAta: interestVault.ataAddress,
                fromAta: await getAssociatedTokenAddress(mint.address, userWallet.publicKey),

                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([userKeyPair])
            .instruction()

        let tx = new Transaction();
        tx.add(ix)

        let hash = await sendAndConfirmTransaction(connection, tx, [userKeyPair], {skipPreflight: true, commitment: 'confirmed'})
        console.log("Your pre-interest signature", hash);

        ix = await program.methods.applyInterest()
            .accounts({
                signer: userKeyPair.publicKey,
                vault: vaultKey,
                toAta: interestVault.ataAddress,
                fromAta: await getAssociatedTokenAddress(mint.address, userWallet.publicKey),
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([userKeyPair])
            .instruction()

        console.log(ix.data.toString("hex"))

        tx = new Transaction();
        tx.add(ix)

        hash = await sendAndConfirmTransaction(connection, tx, [userKeyPair], {skipPreflight: true, commitment: 'confirmed'})

        console.log("Your post-interest signature", hash);

        const bal = await program.account.interestVault.fetch(vaultKey)
        expect(bal.balance.toNumber()).to.eq(101000)
    })
});

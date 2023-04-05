//@ts-nocheck
import { Connection, PublicKey } from "@solana/web3.js";
import { PerpetualsClient } from "./client";
import { createAssociatedTokenAccount, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import poolConfigs from './PoolConfig.json';
import {
    setProvider,
    Program,
    AnchorProvider,
    workspace,
    utils,
    BN,
  } from "@project-serum/anchor";
  import { Perpetuals } from "../../target/types/perpetuals";
  import { LimitOrderCpi } from "../../target/types/limit_order_cpi";

  import {
    PublicKey,
    TransactionInstruction,
    Transaction,
    SystemProgram,
    AccountMeta,
    Keypair,
    SYSVAR_RENT_PUBKEY,
  } from "@solana/web3.js";
  import {
    getAccount,
    getAssociatedTokenAddress,
    createAssociatedTokenAccountInstruction,
    createCloseAccountInstruction,
    createSyncNativeInstruction,
    TOKEN_PROGRAM_ID,
  } from "@solana/spl-token";
  import JSBI from "jsbi";
  import fetch from "node-fetch";
  import { sha256 } from "js-sha256";
  import { encode } from "bs58";
  import { readFileSync } from "fs";
  import { resolveOrCreateAssociatedTokenAddress } from "@orca-so/sdk";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";

const perp_programId = new PublicKey("FastuHRd9PRiuKGy2dHgH4zcSRjARKnPcHiQZnxpR5fD")

 const programId = new PublicKey("41Af5KuLs3fQobV1Pn4q39LGw3aDwY9SWQ4Sj5rB4ZjE")
 const flashProgramId = new PublicKey("FastuHRd9PRiuKGy2dHgH4zcSRjARKnPcHiQZnxpR5fD")
const USDC_MINT =  new PublicKey("Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr");
const BTC_MINT =  new PublicKey("B8DYqbh57aEPRbUq7reyueY6jaYoN75js5YsiM84tFfP");

const findProgramAddress = (label: string, extraSeeds = null) => {
    let seeds = [Buffer.from(utils.bytes.utf8.encode(label))];
    if (extraSeeds) {
      for (let extraSeed of extraSeeds) {
        if (typeof extraSeed === "string") {
          seeds.push(Buffer.from(utils.bytes.utf8.encode(extraSeed)));
        } else if (Array.isArray(extraSeed)) {
          seeds.push(Buffer.from(extraSeed));
        } else {
          seeds.push(extraSeed.toBuffer());
        }
      }
    }
    let res = PublicKey.findProgramAddressSync(seeds, programId);
    return { publicKey: res[0], bump: res[1] };
  };
         //   let pda_key = findProgramAddressSync(
            //     [
            //       Buffer.from("PdaAccount") ,
            //       publicKey.toBuffer(),
            //       POOL_CONFIG.poolAddress.toBuffer(),
            //       payTokenCustody.custodyAccount.toBuffer(),
            //       isVariant(side, 'long') ?  Buffer.from([1]) :  Buffer.from([2]), // in base58 1=2 , 2=3 
            //     ],
            //     perpetual_program.programId
            //   )[0];



// let perp_program, cpi_program  ;
const main = async () => {
    
        try {
            console.log(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>")
                const url = "https://api.devnet.solana.com";
                const adminKey = "/Users/aw/.config/solana/id.json"; // 5H4Dy1KvmAzS228ggSk57CzAqA45sZPpwoV42jyHAXac
                // const adminKey2 = "/Users/aw/.config/solana/Beta-Hcik.json"; // hick


                process.env["ANCHOR_WALLET"] = adminKey;
                // client = new PerpetualsClient(clusterUrl, adminKeyPath);
                console.log("Client Initialized");

               const provider = AnchorProvider.local(url, {
                commitment: "confirmed",
                preflightCommitment: "confirmed",
                skipPreflight: true
              });
              const admin = Keypair.fromSecretKey(
                new Uint8Array(JSON.parse(readFileSync(adminKey).toString()))
              );
            //   const admin2 = Keypair.fromSecretKey(
            //     new Uint8Array(JSON.parse(readFileSync(adminKey2).toString()))
            //   );
              setProvider(provider);
              const perp_program = workspace.Perpetuals as Program<Perpetuals>;
            //   console.log("perp_program:",perp_program)
              const cpi_program = workspace.LimitOrderCpi as Program<LimitOrderCpi>;
            //   console.log("cpi_program:",cpi_program)

            //   const userAddress = admin.publicKey.toBase58();
            //   console.log("userAddress:",userAddress) // 5H4Dy1KvmAzS228ggSk57CzAqA45sZPpwoV42jyHAXac
            //   let pda = findProgramAddress("PdaAccount", [admin.publicKey]) // 8YQqRr7umZXZDsvQTL29fKfhDHnL8pfoszCzANfGqXoa
            //   console.log("pda :",pda.publicKey.toBase58())

            const pdaPubKey = new PublicKey("8YQqRr7umZXZDsvQTL29fKfhDHnL8pfoszCzANfGqXoa")
              console.log("pda :",pdaPubKey.toBase58())
     

            //   const trx = await cpi_program.methods
            //     .initialize()
            //     .accounts({
            //         user : admin.publicKey,
            //         pdaAccount : pda.publicKey,
            //         systemProgram: SystemProgram.programId,
            //         // tokenProgram: TOKEN_PROGRAM_ID,
            //     })
            //     // .remainingAccounts(adminMetas)
            //     .rpc()
            //     .catch((err) => {
            //         console.error(err);
            //         throw err;
            //     });

            const user_usdc_vault = new PublicKey('8LbCQsNBdaSLMvWhEfjcKp36sTQGA1edoXX4498WAWwS');
            const user_btc_vault = new PublicKey('CXtc3H1eog15pgoWtoKhygfGDn6ty8TMEDCRQL9AxCYv');

            // const pda_usdc_vault = findProgramAddress("pda_token_account", [pda.publicKey , USDC_MINT]) 
            // const pda_usdc_vault = await getAssociatedTokenAddress(USDC_MINT,pda.publicKey,true)
            // console.log("pda_usdc_vault:",pda_usdc_vault.publicKey.toBase58())

            // const pda_usdc_vault_new =  await createAssociatedTokenAccount(new Connection(url),admin, USDC_MINT, pda.publicKey)
            // console.log("pda_usdc_vault_new:",pda_usdc_vault_new.toBase58())

            // let pdaTokenAccount = (
            //     await getOrCreateAssociatedTokenAccount(
            //       provider.connection,
            //       admin,
            //       BTC_MINT,
            //       pda.publicKey,
            //       true
            //     )
            //   ).address;
            let pdaTokenAccount = new PublicKey("DUxWbyHbBm1bxme2PMNQ9B2juJK9TQB5RGmgxSDwnqQW")
              console.log("pdaTokenAccount:",pdaTokenAccount.toBase58()) 
              // usdc_vault AS7cJTPEtTWuGfX7BwUnruHNBFvsaP8mK1XPsS5Yb8pB
            //    pda_btc_vault = DUxWbyHbBm1bxme2PMNQ9B2juJK9TQB5RGmgxSDwnqQW



            // const trx = await cpi_program.methods
            //     .deposit({ 
            //         amount : "1000000"
            //     })
            //     .accounts({
            //         user : admin.publicKey,
            //         pdaAccount : pda.publicKey,
            //         tokenMint : USDC_MINT,
            //         ownerTokenVault : user_usdc_vault,
            //         pdaTokenVault : pda_usdc_vault.publicKey,
            //         tokenProgram: TOKEN_PROGRAM_ID,
            //         systemProgram: SystemProgram.programId,
            //     })
            //     // .remainingAccounts(adminMetas)
            //     .rpc()
            //     .catch((err) => {
            //         console.error(">>found error in rpc()",err);
            //         throw err;
            //     });

            // > openPosition inputs 100000 200000 28604198750 31464618625 BTC Object true

            const params: any = {
                price: new BN(31464618625),
                collateral: new BN(100000),
                size: new BN(200000),
                side:  { long: {} },
              };
              const transferAuthorityAddress = new PublicKey("HgM32odRRY36KSmGjAdypSuN5e4QnRdGqQMXMz3Vk26L");
              const perpetualsAddress = new PublicKey("23APwx3K4h7b2FwVYPCc6QUhjxq1HnfiLF5gJGzjjBug");
              const poolAddress = new PublicKey("C8b3A5vcYjkYAT29z9oaa2PiEvvA6qerLEgZJ8Eg3PSE");
              const btcCustody = new PublicKey("463aUWTjWeknez64bRum49Yb2U6PPN9ZyMLnED18Pa3s")

              let positionAccount = findProgramAddressSync(
                [
                  Buffer.from("position") ,
                  pdaPubKey.toBuffer(),
                  poolAddress.toBuffer(),
                  btcCustody.toBuffer(),
                  Buffer.from([1]) 
                ],
                 perp_programId
              )[0];
              console.log("positionAccount:",positionAccount.toBase58())

            const trx = await cpi_program.methods
                .processMarketOrder(params)
                .accounts({
                    keeper: admin.publicKey,
                    user: new PublicKey("5H4Dy1KvmAzS228ggSk57CzAqA45sZPpwoV42jyHAXac"),//waste 
                    pdaAccount : pdaPubKey,
                    pdaTokenVault : pdaTokenAccount,
                    transferAuthority: transferAuthorityAddress,
                    perpetuals: perpetualsAddress,
                    pool: poolAddress,
                    position: positionAccount,
                    custody: btcCustody,
                    custodyOracleAccount: new PublicKey("HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J"),
                    custodyTokenAccount: new PublicKey("DJQM6vZo9m1GBLXFw2j462PkZvakcyojfh1998ZjCddu"),
                    systemProgram: SystemProgram.programId,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    flashProgram : perp_programId
                  })
                // .remainingAccounts(adminMetas)
                .rpc()
                .catch((err) => {
                    console.error(">>found error in rpc()",err);
                    throw err;
                });

                console.log("trx :",`https://explorer.solana.com/tx/${trx}?cluster=devnet`)

            
            console.log(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>")
        } catch (error) {
            console.error("caught error ::: ", error)
        }
    
    console.log("done >>>>")
}
main();



//@ts-nocheck
import { Connection, PublicKey } from "@solana/web3.js";
import { PerpetualsClient } from "./client";
import {
    createAssociatedTokenAccount,
    getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import poolConfigs from "./PoolConfig.json";
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

const perp_programId = new PublicKey(
    "FastuHRd9PRiuKGy2dHgH4zcSRjARKnPcHiQZnxpR5fD"
);

const programId = new PublicKey("41Af5KuLs3fQobV1Pn4q39LGw3aDwY9SWQ4Sj5rB4ZjE");
const flashProgramId = new PublicKey(
    "FastuHRd9PRiuKGy2dHgH4zcSRjARKnPcHiQZnxpR5fD"
);
const USDC_MINT = new PublicKey("Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr");
const BTC_MINT = new PublicKey("B8DYqbh57aEPRbUq7reyueY6jaYoN75js5YsiM84tFfP");

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


// let perp_program, cpi_program  ;
const main = async () => {
    try {
        console.log(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
        const url = "https://api.devnet.solana.com";
        const adminKey = "/Users/aw/.config/solana/id.json"; // 5H4Dy1KvmAzS228ggSk57CzAqA45sZPpwoV42jyHAXac
        // const adminKey2 = "/Users/aw/.config/solana/Beta-Hcik.json"; // hick
        process.env["ANCHOR_WALLET"] = adminKey;

        const provider = AnchorProvider.local(url, {
            commitment: "confirmed",
            preflightCommitment: "confirmed",
            skipPreflight: true,
        });
        setProvider(provider);

        const admin = Keypair.fromSecretKey(
            new Uint8Array(JSON.parse(readFileSync(adminKey).toString()))
        );
        //   const admin2 = Keypair.fromSecretKey(
        //     new Uint8Array(JSON.parse(readFileSync(adminKey2).toString()))
        //   );
        const perp_program = workspace.Perpetuals as Program<Perpetuals>;
        const cpi_program = workspace.LimitOrderCpi as Program<LimitOrderCpi>;

        const userAddress = admin.publicKey.toBase58();
        console.log("userAddress:", userAddress); // 5H4Dy1KvmAzS228ggSk57CzAqA45sZPpwoV42jyHAXac

        let pda = findProgramAddress("PdaDirect1"); // DGhGTk6Mbbaf4Gy9sAK5aen25qqkhKZKkzKaU3wSvhLC 255
        console.log("pda  - PdaDirect1 :", pda.publicKey.toBase58(), pda.bump);

        let pdaTokenAccount = (
            await getOrCreateAssociatedTokenAccount(
                provider.connection,
                admin,
                BTC_MINT,
                pda.publicKey,
                true
            )
        ).address;
        console.log("pdaTokenAccount:", pdaTokenAccount.toBase58());
        // usdc_vault
        //  pda_btc_vault = 4o2Uq1AZ3uiMUvxbK5xQa5PyEKNXB6eceb78hLtH4xG5

        // > openPosition inputs 100000 200000 28604198750 31464618625 BTC Object true

        const params: any = {
            price: new BN(31464618625),
            collateral: new BN(100000),
            size: new BN(200000),
            side: { long: {} },
        };
        const transferAuthorityAddress = new PublicKey(
            "HgM32odRRY36KSmGjAdypSuN5e4QnRdGqQMXMz3Vk26L"
        );
        const perpetualsAddress = new PublicKey(
            "23APwx3K4h7b2FwVYPCc6QUhjxq1HnfiLF5gJGzjjBug"
        );
        const poolAddress = new PublicKey(
            "C8b3A5vcYjkYAT29z9oaa2PiEvvA6qerLEgZJ8Eg3PSE"
        );
        const btcCustody = new PublicKey(
            "463aUWTjWeknez64bRum49Yb2U6PPN9ZyMLnED18Pa3s"
        );

        let positionAccount = findProgramAddressSync(
            [
                Buffer.from("position"),
                pda.publicKey.toBuffer(),
                poolAddress.toBuffer(),
                btcCustody.toBuffer(),
                Buffer.from([1]),
            ],
            perp_programId
        )[0];
        console.log("positionAccount:", positionAccount.toBase58());

        const trx = await cpi_program.methods
            .processMarketOrder(params)
            .accounts({
                keeper: admin.publicKey,
                pdaAccount: pda.publicKey,
                pdaTokenVault: pdaTokenAccount,
                transferAuthority: transferAuthorityAddress,
                perpetuals: perpetualsAddress,
                pool: poolAddress,
                position: positionAccount,
                custody: btcCustody,
                custodyOracleAccount: new PublicKey(
                    "HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J"
                ),
                custodyTokenAccount: new PublicKey(
                    "DJQM6vZo9m1GBLXFw2j462PkZvakcyojfh1998ZjCddu"
                ),
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID,
                flashProgram: perp_programId,
            })
            // .remainingAccounts(adminMetas)
            .rpc()
            .catch((err) => {
                console.error(">>found error in rpc()", err);
                // throw err;
            });
        console.log(
            "trx :",
            `https://explorer.solana.com/tx/${trx}?cluster=devnet`
        );


        console.log(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
    } catch (error) {
        console.error("out caught error ::: ", error);
    }

    console.log("done >>>>");
};
main();

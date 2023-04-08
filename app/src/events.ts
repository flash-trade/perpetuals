//@ts-nocheck
import { Connection, PublicKey } from "@solana/web3.js";
import {
    setProvider,
    Program,
    AnchorProvider,
    workspace,
    utils,
    BN,
    EventParser,
    BorshCoder
} from "@project-serum/anchor";
import { Perpetuals } from "../../target/types/perpetuals";
import {
    PublicKey,
    Transaction,
    Keypair,
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





// let perp_program, cpi_program  ;
const main = async () => {
    try {
        console.log(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
        const url = "https://api.devnet.solana.com";
        const adminKey = "/Users/aw/.config/solana/id.json"; // 5H4Dy1KvmAzS228ggSk57CzAqA45sZPpwoV42jyHAXac
        // const adminKey2 = "/Users/aw/.config/solana/Beta-Hcik.json"; // hcik
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
        const program = workspace.Perpetuals as Program<Perpetuals>;

        const userAddress = admin.publicKey.toBase58();
        console.log("userAddress:", userAddress);

        let pda = findProgramAddress("PdaDirect1"); 
        console.log("pda  - PdaDirect1 :", pda.publicKey.toBase58(), pda.bump);

        // Get transaction from its signature
        const signature = ""; // add signature here !!
        const tx = await provider.connection.getTransaction(signature, {
            commitment: "confirmed",
        });
        console.log(
            "trx :",
            `https://explorer.solana.com/tx/${signature}?cluster=devnet`
        );

        const eventParser = new EventParser(program.programId, new BorshCoder(program.idl));
        const events = eventParser.parseLogs(tx.meta.logMessages);
        console.log("events len :",events.length);
        for (let event of events) {
        console.log(event);
        }

        
        console.log(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
    } catch (error) {
        console.error("out caught error ::: ", error);
    }

    console.log("done >>>>");
};
main();

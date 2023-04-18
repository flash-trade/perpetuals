// JfgUksPtrp3ZvmYQjh5HJWy8kmDV7J4ufxFGM6N8FWHpPbhhQpJZYuZNpmBpWKnsP6UlVHIW3QgP+kh43oT0L/MAQAAAAAAAAD6pGkBAAAAABOWIwAAAAAAJyxHAAAAAAAB

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
import { clusterApiUrl } from "@solana/web3.js";
import { Perpetuals } from "../../target/types/perpetuals";
require('dotenv').config()
const main = async () => { 

    const program = workspace.Perpetuals as Program<Perpetuals>;

    const provider = AnchorProvider.local(clusterApiUrl('devnet'), {
        commitment: "confirmed",
        preflightCommitment: "confirmed",
        skipPreflight: true,
    });

    setProvider(provider);

    const somethign = await provider.connection.getParsedTransaction('21NSFd4qf93BiLQGqMip7YdhtVDaFN6DH1nLTDzvgimrG1ifxXVZgkMCUWUKT56SZ6rAkCtCqsB84S965sZEsjqY')

    console.log('    somethign.meta.logMessages :>> ',     somethign.meta.logMessages);
    const eventParser = new EventParser(program.programId, new BorshCoder(program.idl));

    const events = eventParser.parseLogs(somethign.meta.logMessages)
    for (const e of events) {
        console.log('e :>> ', e);
    }
}

main()
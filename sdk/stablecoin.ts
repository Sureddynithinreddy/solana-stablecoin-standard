import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, Program, Idl } from "@coral-xyz/anchor";
import { Connection, PublicKey, Keypair, Transaction, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import idl from "./my_solana_project.json";
import { STABLECOIN_PROGRAM_ID } from "./constants";

export enum Presets {
    SSS_1 = "SSS-1",
    SSS_2 = "SSS-2",
}

export interface StablecoinOptions {
    preset?: Presets;
    name: string;
    symbol: string;
    uri: string;
    decimals: number;
    authority: Keypair;
    treasury?: PublicKey;
    mint?: PublicKey;
    extensions?: {
        permanentDelegate: boolean;
        transferHook: boolean;
        defaultAccountFrozen: boolean;
    };
}

export class SolanaStablecoin {
    public program: Program;
    public config: PublicKey;
    public mint: PublicKey;

    constructor(program: Program, config: PublicKey, mint: PublicKey) {
        this.program = program;
        this.config = config;
        this.mint = mint;
    }

    static async create(connection: Connection, options: StablecoinOptions): Promise<SolanaStablecoin> {
        const wallet = new anchor.Wallet(options.authority);
        const provider = new AnchorProvider(connection, wallet, {
            commitment: "confirmed",
        });
        const program = new Program(idl as any, STABLECOIN_PROGRAM_ID, provider);

        const configKeypair = Keypair.generate();
        const config = configKeypair.publicKey;
        
        // In real world, we'd create the mint here with extensions if SSS-2
        // For this SDK wrapper, we assume the mint exists or is passed in
        const mint = options.mint || PublicKey.default; 
        const treasury = options.treasury || PublicKey.default;

        let permanentDelegate = false;
        let transferHook = false;
        let defaultAccountFrozen = false;

        if (options.preset === Presets.SSS_2) {
            permanentDelegate = true;
            transferHook = true;
        } else if (options.extensions) {
            permanentDelegate = options.extensions.permanentDelegate;
            transferHook = options.extensions.transferHook;
            defaultAccountFrozen = options.extensions.defaultAccountFrozen;
        }

        await program.methods
            .initializeStablecoin(
                options.name,
                options.symbol,
                options.uri,
                options.decimals,
                permanentDelegate,
                transferHook,
                defaultAccountFrozen
            )
            .accounts({
                config: config,
                admin: options.authority.publicKey,
                mint: mint,
                treasury: treasury,
                systemProgram: SystemProgram.programId,
            })
            .signers([options.authority, configKeypair])
            .rpc();

        return new SolanaStablecoin(program, config, mint);
    }

    async mint(args: { recipient: PublicKey; amount: number; minter: Keypair }): Promise<string> {
        return await this.program.methods
            .mintTokens(new anchor.BN(args.amount))
            .accounts({
                config: this.config,
                admin: args.minter.publicKey,
                mint: this.mint,
                userTokenAccount: args.recipient,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([args.minter])
            .rpc();
    }

    async burn(args: { from: PublicKey; amount: number; user: Keypair }): Promise<string> {
        return await this.program.methods
            .burnTokens(new anchor.BN(args.amount))
            .accounts({
                config: this.config,
                user: args.user.publicKey,
                mint: this.mint,
                userTokenAccount: args.from,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([args.user])
            .rpc();
    }

    async freeze(account: PublicKey, admin: Keypair): Promise<string> {
        return await this.program.methods
            .freezeAccount()
            .accounts({
                config: this.config,
                admin: admin.publicKey,
                mint: this.mint,
                userTokenAccount: account,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([admin])
            .rpc();
    }

    async thaw(account: PublicKey, admin: Keypair): Promise<string> {
        return await this.program.methods
            .thawAccount()
            .accounts({
                config: this.config,
                admin: admin.publicKey,
                mint: this.mint,
                userTokenAccount: account,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([admin])
            .rpc();
    }

    async pause(admin: Keypair): Promise<string> {
        return await this.program.methods
            .pause()
            .accounts({ config: this.config, admin: admin.publicKey })
            .signers([admin])
            .rpc();
    }

    async unpause(admin: Keypair): Promise<string> {
        return await this.program.methods
            .unpause()
            .accounts({ config: this.config, admin: admin.publicKey })
            .signers([admin])
            .rpc();
    }

    public compliance = {
        blacklistAdd: async (wallet: PublicKey, admin: Keypair): Promise<string> => {
            const [blacklistPDA] = PublicKey.findProgramAddressSync(
                [Buffer.from("blacklist"), wallet.toBuffer()],
                this.program.programId
            );
            return await this.program.methods
                .addToBlacklist(wallet)
                .accounts({
                    config: this.config,
                    blacklist: blacklistPDA,
                    admin: admin.publicKey,
                    systemProgram: SystemProgram.programId,
                })
                .signers([admin])
                .rpc();
        },
        blacklistRemove: async (wallet: PublicKey, admin: Keypair): Promise<string> => {
            const [blacklistPDA] = PublicKey.findProgramAddressSync(
                [Buffer.from("blacklist"), wallet.toBuffer()],
                this.program.programId
            );
            return await this.program.methods
                .removeFromBlacklist()
                .accounts({
                    config: this.config,
                    blacklist: blacklistPDA,
                    admin: admin.publicKey,
                })
                .signers([admin])
                .rpc();
        },
        seize: async (from: PublicKey, to: PublicKey, amount: number, admin: Keypair): Promise<string> => {
            return await this.program.methods
                .seize(new anchor.BN(amount))
                .accounts({
                    config: this.config,
                    admin: admin.publicKey,
                    mint: this.mint,
                    userTokenAccount: from,
                    treasuryTokenAccount: to,
                    tokenProgram: TOKEN_PROGRAM_ID,
                })
                .signers([admin])
                .rpc();
        }
    };

    async updateMinter(newMinter: PublicKey, admin: Keypair): Promise<string> {
        return await this.program.methods
            .updateMinter(newMinter)
            .accounts({ config: this.config, admin: admin.publicKey })
            .signers([admin])
            .rpc();
    }

    async transferAuthority(newAdmin: PublicKey, admin: Keypair): Promise<string> {
        return await this.program.methods
            .transferAuthority(newAdmin)
            .accounts({ config: this.config, admin: admin.publicKey })
            .signers([admin])
            .rpc();
    }
    
    async getTotalSupply(): Promise<number> {
        const mintInfo = await this.program.provider.connection.getTokenSupply(this.mint);
        return Number(mintInfo.value.amount);
    }
}
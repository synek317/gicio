import type { FileDiff } from './types';

/** The change touches Rust code, so the cargo tools apply. */
export const touchesRust = (files: FileDiff[]) => files.some((f) => f.path.endsWith('.rs') || f.path.endsWith('Cargo.toml'));

/** The change touches C# code, so the dotnet tools apply. */
export const touchesCSharp = (files: FileDiff[]) => files.some((f) => /\.(cs|csx|csproj|sln|slnx)$/.test(f.path));

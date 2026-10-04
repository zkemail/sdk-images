// End-to-end check of circuits rendered from templates/template.nr.tera (zkemail.nr v2.0.0).
// Usage: cargo run -p noir --example generate_v2_fixtures -- <dir>
//        PATH=<nargo 1.0.0-beta.5>:<bb 0.84.0>:$PATH node tests/v2-proof-smoke.mjs <dir>
// Uses synthetic RSA keys and synthetic emails only; no private email or production credentials.
import { generateKeyPairSync, sign, createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';
import assert from 'node:assert/strict';

const root = resolve(process.argv[2]);
const SCHEME = ['--scheme', 'ultra_honk', '--oracle_hash', 'keccak'];
// Public prefix: modulus hash, redc hash, nullifier, header_hash[0..2], prover_address.
const PREFIX = 6;

function run(command, args, cwd, succeeds = true) {
  const result = spawnSync(command, args, { cwd, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024, timeout: 300000 });
  const output = `${result.stdout || ''}${result.stderr || ''}`;
  if (result.error) throw result.error;
  if (succeeds) assert.equal(result.status, 0, output);
  else assert.notEqual(result.status, 0, `invalid input unexpectedly accepted: ${args.join(' ')}`);
  return output;
}
const limbs = (value, bits) => Array.from({ length: Math.ceil(bits / 120) }, () => {
  const limb = value & ((1n << 120n) - 1n);
  value >>= 120n;
  return limb.toString();
});
const integer = bytes => BigInt(`0x${Buffer.from(bytes).toString('hex')}`);
const padded = (bytes, size) => [...bytes, ...Array(size - bytes.length).fill(0)];
const toml = x => JSON.stringify(x);

function rsaKey(bits) {
  const { privateKey, publicKey } = generateKeyPairSync('rsa', { modulusLength: bits, publicExponent: 65537 });
  const modulus = integer(Buffer.from(publicKey.export({ format: 'jwk' }).n, 'base64url'));
  // noir-bignum v0.6.0 / noir_rsa v0.7.0 reduction parameter.
  const redc = (1n << BigInt(2 * bits + 4)) / modulus;
  const pubkey = `[pubkey]\nmodulus = ${toml(limbs(modulus, bits))}\nredc = ${toml(limbs(redc, bits))}\n`;
  return { bits, privateKey, pubkey };
}
const signatureFor = (key, header) => limbs(integer(sign('sha256', header, key.privateKey)), key.bits);

// Execute a named Prover file; on success, prove and verify and return the public inputs.
function proveAndVerify(dir, name) {
  run('nargo', ['execute', name, '--prover-name', name], dir);
  const out = join(dir, `proof-${name}`); mkdirSync(out, { recursive: true });
  run('bb', ['prove', '-b', join(dir, 'target/sdk_noir.json'), '-w', join(dir, `target/${name}.gz`), '-o', out,
    '--output_format', 'bytes_and_fields', '--write_vk', ...SCHEME], dir);
  run('bb', ['verify', '-p', join(out, 'proof'), '-k', join(out, 'vk'), '-i', join(out, 'public_inputs'), ...SCHEME], dir);
  return { out, inputs: JSON.parse(readFileSync(join(out, 'public_inputs_fields.json'), 'utf8')).map(BigInt) };
}
const rejects = (dir, name) => run('nargo', ['execute', name, '--prover-name', name], dir, false);

function checkPrefix(inputs, header) {
  assert.notEqual(inputs[0], 0n, 'modulus hash'); assert.notEqual(inputs[1], 0n, 'redc hash');
  assert.notEqual(inputs[2], 0n, 'nullifier'); assert.equal(inputs[5], 1n, 'prover address');
  const digest = createHash('sha256').update(header).digest();
  for (let i = 0; i < 2; i++) {
    // zkemail.nr pack_bytes is big-endian within each 16-byte element.
    const packed = [...digest.subarray(i * 16, (i + 1) * 16)].reduce((a, byte) => (a << 8n) + BigInt(byte), 0n);
    assert.equal(inputs[3 + i], packed, 'header digest positions');
  }
}
const bytesOf = fields => fields.map(Number);

// Header-only circuits (header masking, no body), both key sizes.
for (const bits of [1024, 2048]) {
  const dir = join(root, String(bits)); const key = rsaKey(bits); const MAX = 128;
  const header = Buffer.from('from:alice@example.com\r\nsubject: SDK v2 test\r\n');
  const storage = padded(header, MAX); const mask = storage.map(() => true); // honest "reveal everything"
  const signature = signatureFor(key, header);
  const write = (name, { bytes = storage, sig = signature } = {}) => writeFileSync(join(dir, `${name}.toml`),
    `signature = ${toml(sig)}\nprover_address = ["1"]\nheader_mask = ${toml(mask)}\n[header]\nlen = ${header.length}\nstorage = ${toml(bytes)}\n${key.pubkey}`);
  run('nargo', ['compile'], dir);

  write('Valid');
  const { out, inputs } = proveAndVerify(dir, 'Valid');
  assert.equal(inputs.length, PREFIX + MAX, 'six prefix fields plus masked header bytes');
  checkPrefix(inputs, header);
  assert.deepEqual(bytesOf(inputs.slice(PREFIX)), storage);

  const badHeader = [...storage]; badHeader[0] ^= 1; write('BadHeader', { bytes: badHeader });
  rejects(dir, 'BadHeader');
  write('BadSignature', { sig: signature.map((x, i) => (i ? x : (BigInt(x) ^ 1n).toString())) });
  rejects(dir, 'BadSignature');

  // Unsigned text in storage past header.len() must never be published, even with an all-true mask.
  const tail = [...storage];
  Buffer.from('subject: FORGED').forEach((b, i) => { tail[header.length + i] = b; });
  write('UnsignedTail', { bytes: tail });
  const forged = proveAndVerify(dir, 'UnsignedTail').inputs;
  assert.deepEqual(bytesOf(forged.slice(PREFIX)), storage, 'unsigned header tail was published');

  // Public-input tampering: modulus hash, redc hash and prover address are bound by the proof.
  for (const index of [0, 1, 5]) {
    const corrupted = Buffer.from(readFileSync(join(out, 'public_inputs')));
    corrupted[index * 32 + 31] ^= 1;
    const path = join(out, `tampered-${index}`); writeFileSync(path, corrupted);
    run('bb', ['verify', '-p', join(out, 'proof'), '-k', join(out, 'vk'), '-i', path, ...SCHEME], dir, false);
  }
  // NOTE: v2 exposes the redc hash publicly instead of proving redc is correct for the modulus;
  // a perturbed redc can still yield a witness, so verifiers must authorize BOTH key hashes.
  console.log(`${bits}: proof verified; tampered header/signature rejected; unsigned tail hidden; key-hash/prover tampering rejected`);
}

// Body circuit: body-hash check + soft line break removal + body masking (1024-bit key).
{
  const dir = join(root, 'body'); const key = rsaKey(1024); const HMAX = 256, BMAX = 128;
  const body = Buffer.from('hello =\r\nworld\r\n');
  const decoded = Buffer.from('hello world\r\n');
  const bh = createHash('sha256').update(body).digest('base64');
  const prefix = 'from:alice@example.com\r\nsubject: SDK v2 body test\r\n';
  const header = Buffer.from(`${prefix}dkim-signature:v=1; a=rsa-sha256; d=example.com; bh=${bh}; b=`);
  const fieldIndex = prefix.length;
  const bhIndex = header.indexOf(`bh=${bh}`) + 3;
  const signature = signatureFor(key, header);
  const sha256Iv = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
  const write = (name, { bodyBytes = padded(body, BMAX), decodedBytes = padded(decoded, BMAX), decodedLen = decoded.length } = {}) =>
    writeFileSync(join(dir, `${name}.toml`), [
      `signature = ${toml(signature)}`, 'prover_address = ["1"]',
      `header_mask = ${toml(Array(HMAX).fill(true))}`, `body_mask = ${toml(Array(BMAX).fill(true))}`,
      `body_hash_index = ${bhIndex}`, `partial_body_hash = ${toml(sha256Iv)}`, `partial_body_real_length = ${body.length}`,
      `[header]\nlen = ${header.length}\nstorage = ${toml(padded(header, HMAX))}`,
      `[dkim_header_sequence]\nindex = ${fieldIndex}\nlength = ${header.length - fieldIndex}`,
      `[body]\nlen = ${body.length}\nstorage = ${toml(bodyBytes)}`,
      `[decoded_body]\nlen = ${decodedLen}\nstorage = ${toml(decodedBytes)}`, key.pubkey].join('\n'));
  run('nargo', ['compile'], dir);

  write('Valid');
  const { inputs } = proveAndVerify(dir, 'Valid');
  assert.equal(inputs.length, PREFIX + HMAX + BMAX);
  checkPrefix(inputs, header);
  assert.deepEqual(bytesOf(inputs.slice(PREFIX, PREFIX + HMAX)), padded(header, HMAX));
  assert.deepEqual(bytesOf(inputs.slice(PREFIX + HMAX)), padded(decoded, BMAX));

  // Honest non-zero bytes past body.len() (e.g. SHA padding with sha_precompute_selector) are ignored.
  const junkTail = padded(body, BMAX); Buffer.from('EVIL').forEach((b, i) => { junkTail[body.length + i] = b; });
  write('JunkTail', { bodyBytes: junkTail });
  assert.deepEqual(bytesOf(proveAndVerify(dir, 'JunkTail').inputs.slice(PREFIX + HMAX)), padded(decoded, BMAX));

  // ...but they cannot be smuggled into decoded_body (and from there into body masks/regexes).
  const forgedDecoded = padded(decoded, BMAX); Buffer.from('EVIL').forEach((b, i) => { forgedDecoded[decoded.length + i] = b; });
  write('ForgedDecoded', { bodyBytes: junkTail, decodedBytes: forgedDecoded, decodedLen: decoded.length + 4 });
  rejects(dir, 'ForgedDecoded');

  const badBody = padded(body, BMAX); badBody[0] ^= 1; write('BadBody', { bodyBytes: badBody });
  rejects(dir, 'BadBody');
  console.log('body: proof verified; body-hash tampering rejected; unsigned body tail ignored and cannot reach decoded_body');
}

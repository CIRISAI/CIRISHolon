# DEVICES-0 — the record made before any survey row is parsed (2026-11-11)

- **Plants.** All eight pass on synthetic households (`plants.log`).
- **Files on this machine.** Both years' microdata, both codebooks and both user guides;
  digests in `raw_file_sha256.txt`. Looked at so far: the header line of each data file
  (column names) and the codebook entries for the columns read. No data row.
- **The error method, read from the agency's guide:** 60 jackknife replicates;
  variance = (R − 1) / R × the sum over replicates of (replicate estimate − full estimate)².
- **Codebook facts that differ from the freeze's assumptions, stated before the read:**
  - 2024 names the household columns differently and top-codes more: adults 18 to 64 at
    4, adults over 64 at 2, members derived from the three and running to 10. S2 is
    computed on 2020 only, where adults are not top-coded below 7.
  - 2024's urban type has two values (urban, rural); 2020's has three.
  - Housing type: 2 is a detached single house, 5 an apartment in a building of five or
    more units. Those are S5's two groups.
  - "Not applicable" (−2) in a count is read as zero.
- **Choices the freeze left open:**
  - S2 keeps households with fewer than 7 members and fewer than 4 children (both below
    their top codes).
  - S3's gap is two times the root of the two years' summed variances (two independent
    samples); its budget is 15 % of the 2024 count.
  - S5's size standard is the 2020 mix over sizes 1, 2, 3 and 4 or more; the relative
    difference is over the mean of the two groups.
  - S7 is met if the ratio less two standard errors is at least 1.5, killed if the ratio
    plus two standard errors is under 1.2.
  - The 2015 file is NOT read in this run; S4 has two points and says so.

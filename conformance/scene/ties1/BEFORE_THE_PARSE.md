# TIES-1 — the record made before any data row is parsed (2026-11-11)

- **Plants.** All nine pass on synthetic respondents (`plants.log`).
- **The file** was downloaded by the lead under his own account and moved to scratch by
  this session; its digest is in `raw_file_sha256.txt`. Read so far: the questionnaire
  and the data file's column names. Not read: any row, the topline, the crosstabs.
- **Read from the questionnaire:** adults in the household are recorded as one, two, or
  three or more; children as one, two, three or more, or none. So coverage in S0 counts
  "three or more" as three and is overstated for large households.
- **Choices the freeze left open, stated before the read:**
  - A tie exists for a name slot that has a connection code or a distance.
  - Relation, where several family connections are given: partner, then child, parent,
    sibling, other family. No family code: non-kin.
  - A same-house tie whose relation is "child" counts against children at home (a grown
    child at home is miscounted as a child); any other same-house tie counts against the
    other adults.
  - "Holds the means": mobile voice needs a mobile; text needs a mobile used for texts;
    email needs internet or email use; instant message and social-network message need
    that activity; landline is taken as held by each respondent.
  - The second population is respondents who hold mobile, text and email.
  - S1 is met if the difference less its gap is at least 0.05 on the days scale and at
    each of the four acts scales.
  - S4: the file's `density3` and `usr` columns are described from their own values in
    G0; the stake's form (slope or contrast) is decided from that description and said in
    the results before any share by place is quoted.
  - S7's groups: four or more of mobile voice, email, text, instant message and
    social-network message; two or fewer.
  - S5 (the building, kept from the draft as a reading): detached house against apartment.

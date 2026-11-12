# Policy frames

Twelve frames: 1-6 are phrasing 1, 7-12 are phrasing 2. Frame index for an item = int(sha256(item id).hexdigest()[:8], 16) % 6; phrasing 1 uses frame index + 1 and phrasing 2 its partner, frame index + 7. (Earlier version used item number modulo 6, which tied frame to elapsed time; replaced at the lead's request.)
Final line, phrasing 1: `Has it taken effect, is it still waiting, or has it been dropped?`
Final line, phrasing 2: `Which is it now: dropped, still waiting, or taken effect?`

Slots: {p} what is proposed; {m} members; {c} counted; {a} agreed; {o} objected; {ha}/{ho} has or have; {el} exact elapsed time with 'ago' plus the raw seconds; {age} the same without 'ago'; {w} waiting period in the source's words; {f} the fraction as given ('2 in 3'); {d} objections that drop it; {g} agreements needed at the close.
The rule is assembled as `now` (+ `exc` only when the row's rule has the 'always waits its whole window' exception) + `wait` + `end_num` (or `end_none` when the row's rule gives no number for objections).
`cnt_same` is used when counted equals members, `cnt_diff` when it does not.

## Frame 1 (phrasing 1)

- **body**: `The proposal is {p}. {cnt} So far {a} {ha} agreed and {o} {ho} objected. It was proposed {el}, and its waiting period is {w}. The rule is as follows. {rule}`
- **cnt_same**: `There are {m} members, and all {c} of them are counted.`
- **cnt_diff**: `There are {m} members, of whom {c} are counted.`
- **now**: `It takes effect at once if strictly more than {f} of those counted agree{exc}.`
- **exc**: `, except that this kind of change always waits its whole waiting period`
- **wait**: `Otherwise it waits until its waiting period is over.`
- **end_num**: `When the waiting period ends, it is dropped if at least {d} objected; if it is not dropped, it takes effect provided at least {g} agreed, and is dropped otherwise.`
- **end_none**: `When the waiting period ends, objections do not drop it, whatever their number; it takes effect only if the number needed for taking effect at once has agreed.`

## Frame 2 (phrasing 1)

- **body**: `First the rule. {rule} Now the case. What has been proposed is {p}, and the waiting period is {w}. {cnt} Agreed so far: {a}. Objected so far: {o}. The proposal was made {el}.`
- **cnt_same**: `The group has {m} members and every one of the {c} is counted.`
- **cnt_diff**: `The group has {m} members and {c} of them are counted.`
- **now**: `A proposal takes effect immediately whenever strictly more than {f} of those counted agree{exc}.`
- **exc**: `, though a change of this kind is an exception and always waits out its entire waiting period`
- **wait**: `If that does not happen, the proposal waits until the waiting period has run out.`
- **end_num**: `At that point, {d} or more objections mean it is dropped. If it is not dropped, {g} or more agreements mean it takes effect, and fewer than that mean it is dropped.`
- **end_none**: `At that point, no number of objections causes it to be dropped; it takes effect only if as many have agreed as are needed for it to take effect immediately.`

## Frame 3 (phrasing 1)

- **body**: `A proposal was made {el}. It is {p}. Its waiting period is {w}. {cnt} The count of those who have agreed is {a}; the count of those who have objected is {o}. {rule}`
- **cnt_same**: `There are {m} members and {c} are counted, which is all of them.`
- **cnt_diff**: `There are {m} members and {c} are counted, which is not all of them.`
- **now**: `The rule says the proposal takes effect straight away when strictly more than {f} of those counted agree{exc}.`
- **exc**: `; but for this kind of change there is an exception, and it always waits its whole waiting period`
- **wait**: `Otherwise the proposal waits for its waiting period to end.`
- **end_num**: `Once the waiting period has ended, the proposal is dropped if at least {d} objected. If that does not drop it, it takes effect as long as at least {g} agreed; otherwise it is dropped.`
- **end_none**: `Once the waiting period has ended, objections do not drop the proposal (there is no number of them that does), and it takes effect only if the number needed for taking effect straight away has agreed.`

## Frame 4 (phrasing 1)

- **body**: `{cnt} The proposal in front of them is {p}. {rule} The waiting period for this proposal is {w}, and it was proposed {el}. At this moment {a} {ha} agreed and {o} {ho} objected.`
- **cnt_same**: `A group has {m} members, all {c} of whom are counted.`
- **cnt_diff**: `A group has {m} members, {c} of whom are counted.`
- **now**: `Under the rule, it takes effect at once in the case where strictly more than {f} of those counted agree{exc}.`
- **exc**: `, except that a change of this kind always waits its whole waiting period`
- **wait**: `In any other case it waits out its waiting period.`
- **end_num**: `When that period closes, it is dropped if at least {d} objected. If it has not been dropped, it takes effect if at least {g} agreed and is dropped if they did not.`
- **end_none**: `When that period closes, it is not dropped on account of objections, however many there are; it takes effect only if the number needed for taking effect at once has agreed.`

## Frame 5 (phrasing 1)

- **body**: `Agreed: {a}. Objected: {o}. Those are the numbers so far on a proposal made {el}. The proposal is {p}. {cnt} The waiting period is {w}. Here is the rule that applies. {rule}`
- **cnt_same**: `All {c} of the {m} members are counted.`
- **cnt_diff**: `Of the {m} members, {c} are counted.`
- **now**: `The proposal takes effect at once when strictly more than {f} of those counted agree{exc}.`
- **exc**: `, except that this kind of change always waits the whole of its waiting period`
- **wait**: `When that is not so, it waits until the waiting period is finished.`
- **end_num**: `At the finish, if at least {d} objected, it is dropped. If it is not dropped, it takes effect when at least {g} agreed and is dropped when fewer did.`
- **end_none**: `At the finish, objections do not cause it to be dropped, since no number is set for that; it takes effect only if the number needed for taking effect at once has agreed.`

## Frame 6 (phrasing 1)

- **body**: `The waiting period is {w}. The proposal, made {el}, is {p}. {rule} {cnt} The number who have agreed is {a} and the number who have objected is {o}.`
- **cnt_same**: `Counted are {c} people, which is every one of the {m} members.`
- **cnt_diff**: `Counted are {c} people out of {m} members.`
- **now**: `By the rule, it takes effect at once if strictly more than {f} of those counted agree{exc}.`
- **exc**: `, except that this kind of change always waits its full waiting period`
- **wait**: `If not, it stays waiting until the waiting period is up.`
- **end_num**: `After that, at least {d} having objected means it is dropped; when it is not dropped, at least {g} having agreed means it takes effect, and anything less means it is dropped.`
- **end_none**: `After that, objections do not drop it at any number, and it takes effect only if the number needed for taking effect at once has agreed.`

## Frame 7 (phrasing 2)

- **body**: `Up for decision is {p}. It was put forward {el}, with a waiting time of {w}. {cnt} Those in agreement number {a}; those objecting number {o}. How such a proposal is settled: {rule}`
- **cnt_same**: `The membership is {m}, and the number counted is the same, {c}.`
- **cnt_diff**: `The membership is {m}, and the number counted is {c}.`
- **now**: `if the ones agreeing are strictly more than {f} of those counted, it takes effect on the spot{exc}.`
- **exc**: ` (with one exception: a change of this kind always sits through its whole waiting time)`
- **wait**: `Short of that, it sits through its waiting time.`
- **end_num**: `When the waiting time is over, {d} or more objections get it dropped; without that, {g} or more agreements make it take effect, and fewer than {g} get it dropped.`
- **end_none**: `When the waiting time is over, objections cannot get it dropped, because no number is set for them; it takes effect only if the agreements reach the number needed for taking effect on the spot.`

## Frame 8 (phrasing 2)

- **body**: `{rule} That is the rule. Here it applies to {p}, where the waiting time is {w} and the proposal dates from {el}. {cnt} Agreeing so far: {a}. Objecting so far: {o}.`
- **cnt_same**: `Members number {m}; counted, {c}, so nobody is left out of the count.`
- **cnt_diff**: `Members number {m}; counted, {c}.`
- **now**: `A proposal that has the agreement of strictly more than {f} of those counted takes effect right away{exc}.`
- **exc**: `, unless it is a change of this kind, which always waits its whole waiting time`
- **wait**: `A proposal that does not take effect right away waits for its waiting time to pass.`
- **end_num**: `Then two checks are made in order. First, if at least {d} have objected, it is dropped. Second, if it was not dropped, it takes effect if at least {g} have agreed, and if not, it is dropped.`
- **end_none**: `Then, at the end of the waiting time, objections play no part in dropping it, as no number is given for them; it takes effect only if the number needed for taking effect right away has agreed.`

## Frame 9 (phrasing 2)

- **body**: `It was {el} that the proposal was made. The proposal: {p}. Waiting time: {w}. {cnt} {a} {ha} agreed. {o} {ho} objected. {rule}`
- **cnt_same**: `{m} people are members and all {c} are counted.`
- **cnt_diff**: `{m} people are members and {c} of them are counted.`
- **now**: `According to the rule, agreement by strictly more than {f} of those counted makes a proposal take effect immediately{exc}.`
- **exc**: `, except for a change of this kind, which always waits its whole waiting time`
- **wait**: `Without that, the proposal waits for the full waiting time.`
- **end_num**: `When the waiting time has closed, the proposal is dropped if the objections number at least {d}; if not dropped, it takes effect if the agreements number at least {g}, and is dropped if they do not.`
- **end_none**: `When the waiting time has closed, the proposal is not dropped for objections, there being no number for that; it takes effect only if the agreements reach the number needed for taking effect immediately.`

## Frame 10 (phrasing 2)

- **body**: `Consider a proposal for {p}. {rule} In this case the waiting time is {w} and the proposal is now {age} old. {cnt} So far there are {a} in agreement and {o} objecting.`
- **cnt_same**: `All of the {m} members are counted, making {c} counted.`
- **cnt_diff**: `Out of {m} members, {c} are counted.`
- **now**: `The rule for it: it takes effect without delay if strictly more than {f} of those counted agree{exc}.`
- **exc**: `, except that a change of this kind always waits through all of its waiting time`
- **wait**: `Otherwise it waits through its waiting time.`
- **end_num**: `At the end of that time it is dropped if at least {d} objected; and if it is not dropped, then it takes effect so long as at least {g} agreed, and is dropped when that is not so.`
- **end_none**: `At the end of that time it is not dropped because of objections (no number of objections does that); it takes effect only if the number needed for taking effect without delay has agreed.`

## Frame 11 (phrasing 2)

- **body**: `{cnt} {a} {ha} agreed to a proposal and {o} {ho} objected to it. The proposal is {p}; it was made {el} and carries a waiting time of {w}. What the rule says: {rule}`
- **cnt_same**: `Among {m} members, the number counted is {c}, the whole membership.`
- **cnt_diff**: `Among {m} members, the number counted is {c}.`
- **now**: `with strictly more than {f} of those counted agreeing, the proposal takes effect at once{exc}.`
- **exc**: `; however, this kind of change always waits its whole waiting time`
- **wait**: `If it has not taken effect at once, it waits until the waiting time ends.`
- **end_num**: `At the end, it is dropped where at least {d} objected. Where it is not dropped, it takes effect if at least {g} agreed; if fewer agreed, it is dropped.`
- **end_none**: `At the end, it is not dropped for objections at all, no number being set; it takes effect only if the number needed for taking effect at once has agreed.`

## Frame 12 (phrasing 2)

- **body**: `Since the proposal was made, an interval of {age} has gone by; the waiting time set for it is {w}. What is proposed is {p}. {rule} {cnt} The tally so far is {a} agreed, {o} objected.`
- **cnt_same**: `The members number {m} and the counted number {c}, the same figure.`
- **cnt_diff**: `The members number {m} and the counted number {c}.`
- **now**: `The rule is that it takes effect at once as soon as strictly more than {f} of those counted agree{exc}.`
- **exc**: `, except that this kind of change always waits out its whole waiting time`
- **wait**: `Otherwise it waits out the waiting time.`
- **end_num**: `When the waiting time closes: dropped if at least {d} objected; if not dropped, takes effect if at least {g} agreed; dropped otherwise.`
- **end_none**: `When the waiting time closes: objections do not drop it, since there is no number for that; it takes effect only if the number needed for taking effect at once has agreed.`

## Script: policy_fill.py

Run as `python3 -I policy_fill.py <policy_facts.tsv> <out_dir>`.

```python
#!/usr/bin/env python3
"""Fill the twelve policy frames from policy_facts.tsv.
usage: python3 -I policy_fill.py <policy_facts.tsv> <out_dir>
Frame index = int(sha256(item id).hexdigest()[:8], 16) % 6; phrasing 1 takes frame index+1,
phrasing 2 its partner, frame index+7. The choice depends on the item id alone, not on the
content of the row, and is not tied to the row's position in the time cycle."""
import csv, hashlib, json, os, re, sys

Q1 = "Has it taken effect, is it still waiting, or has it been dropped?"
Q2 = "Which is it now: dropped, still waiting, or taken effect?"

# Each frame: body, cnt_same (members == counted), cnt_diff, now (+exc), wait, end_num, end_none.
# Slots: {p} proposal  {m} members  {c} counted  {a} agreed  {o} objected  {ha}/{ho} has|have
#        {el} elapsed with "ago"  {age} elapsed without "ago"  {w} waiting period
#        {f} the "2 in 3" fraction  {d} number of objections that drops  {g} number of agreements needed
FRAMES = [
 # ---------------- phrasing 1 ----------------
 dict(body="The proposal is {p}. {cnt} So far {a} {ha} agreed and {o} {ho} objected. It was proposed {el}, and its waiting period is {w}. The rule is as follows. {rule}",
      cnt_same="There are {m} members, and all {c} of them are counted.",
      cnt_diff="There are {m} members, of whom {c} are counted.",
      now="It takes effect at once if strictly more than {f} of those counted agree{exc}.",
      exc=", except that this kind of change always waits its whole waiting period",
      wait="Otherwise it waits until its waiting period is over.",
      end_num="When the waiting period ends, it is dropped if at least {d} objected; if it is not dropped, it takes effect provided at least {g} agreed, and is dropped otherwise.",
      end_none="When the waiting period ends, objections do not drop it, whatever their number; it takes effect only if the number needed for taking effect at once has agreed."),
 dict(body="First the rule. {rule} Now the case. What has been proposed is {p}, and the waiting period is {w}. {cnt} Agreed so far: {a}. Objected so far: {o}. The proposal was made {el}.",
      cnt_same="The group has {m} members and every one of the {c} is counted.",
      cnt_diff="The group has {m} members and {c} of them are counted.",
      now="A proposal takes effect immediately whenever strictly more than {f} of those counted agree{exc}.",
      exc=", though a change of this kind is an exception and always waits out its entire waiting period",
      wait="If that does not happen, the proposal waits until the waiting period has run out.",
      end_num="At that point, {d} or more objections mean it is dropped. If it is not dropped, {g} or more agreements mean it takes effect, and fewer than that mean it is dropped.",
      end_none="At that point, no number of objections causes it to be dropped; it takes effect only if as many have agreed as are needed for it to take effect immediately."),
 dict(body="A proposal was made {el}. It is {p}. Its waiting period is {w}. {cnt} The count of those who have agreed is {a}; the count of those who have objected is {o}. {rule}",
      cnt_same="There are {m} members and {c} are counted, which is all of them.",
      cnt_diff="There are {m} members and {c} are counted, which is not all of them.",
      now="The rule says the proposal takes effect straight away when strictly more than {f} of those counted agree{exc}.",
      exc="; but for this kind of change there is an exception, and it always waits its whole waiting period",
      wait="Otherwise the proposal waits for its waiting period to end.",
      end_num="Once the waiting period has ended, the proposal is dropped if at least {d} objected. If that does not drop it, it takes effect as long as at least {g} agreed; otherwise it is dropped.",
      end_none="Once the waiting period has ended, objections do not drop the proposal (there is no number of them that does), and it takes effect only if the number needed for taking effect straight away has agreed."),
 dict(body="{cnt} The proposal in front of them is {p}. {rule} The waiting period for this proposal is {w}, and it was proposed {el}. At this moment {a} {ha} agreed and {o} {ho} objected.",
      cnt_same="A group has {m} members, all {c} of whom are counted.",
      cnt_diff="A group has {m} members, {c} of whom are counted.",
      now="Under the rule, it takes effect at once in the case where strictly more than {f} of those counted agree{exc}.",
      exc=", except that a change of this kind always waits its whole waiting period",
      wait="In any other case it waits out its waiting period.",
      end_num="When that period closes, it is dropped if at least {d} objected. If it has not been dropped, it takes effect if at least {g} agreed and is dropped if they did not.",
      end_none="When that period closes, it is not dropped on account of objections, however many there are; it takes effect only if the number needed for taking effect at once has agreed."),
 dict(body="Agreed: {a}. Objected: {o}. Those are the numbers so far on a proposal made {el}. The proposal is {p}. {cnt} The waiting period is {w}. Here is the rule that applies. {rule}",
      cnt_same="All {c} of the {m} members are counted.",
      cnt_diff="Of the {m} members, {c} are counted.",
      now="The proposal takes effect at once when strictly more than {f} of those counted agree{exc}.",
      exc=", except that this kind of change always waits the whole of its waiting period",
      wait="When that is not so, it waits until the waiting period is finished.",
      end_num="At the finish, if at least {d} objected, it is dropped. If it is not dropped, it takes effect when at least {g} agreed and is dropped when fewer did.",
      end_none="At the finish, objections do not cause it to be dropped, since no number is set for that; it takes effect only if the number needed for taking effect at once has agreed."),
 dict(body="The waiting period is {w}. The proposal, made {el}, is {p}. {rule} {cnt} The number who have agreed is {a} and the number who have objected is {o}.",
      cnt_same="Counted are {c} people, which is every one of the {m} members.",
      cnt_diff="Counted are {c} people out of {m} members.",
      now="By the rule, it takes effect at once if strictly more than {f} of those counted agree{exc}.",
      exc=", except that this kind of change always waits its full waiting period",
      wait="If not, it stays waiting until the waiting period is up.",
      end_num="After that, at least {d} having objected means it is dropped; when it is not dropped, at least {g} having agreed means it takes effect, and anything less means it is dropped.",
      end_none="After that, objections do not drop it at any number, and it takes effect only if the number needed for taking effect at once has agreed."),
 # ---------------- phrasing 2 ----------------
 dict(body="Up for decision is {p}. It was put forward {el}, with a waiting time of {w}. {cnt} Those in agreement number {a}; those objecting number {o}. How such a proposal is settled: {rule}",
      cnt_same="The membership is {m}, and the number counted is the same, {c}.",
      cnt_diff="The membership is {m}, and the number counted is {c}.",
      now="if the ones agreeing are strictly more than {f} of those counted, it takes effect on the spot{exc}.",
      exc=" (with one exception: a change of this kind always sits through its whole waiting time)",
      wait="Short of that, it sits through its waiting time.",
      end_num="When the waiting time is over, {d} or more objections get it dropped; without that, {g} or more agreements make it take effect, and fewer than {g} get it dropped.",
      end_none="When the waiting time is over, objections cannot get it dropped, because no number is set for them; it takes effect only if the agreements reach the number needed for taking effect on the spot."),
 dict(body="{rule} That is the rule. Here it applies to {p}, where the waiting time is {w} and the proposal dates from {el}. {cnt} Agreeing so far: {a}. Objecting so far: {o}.",
      cnt_same="Members number {m}; counted, {c}, so nobody is left out of the count.",
      cnt_diff="Members number {m}; counted, {c}.",
      now="A proposal that has the agreement of strictly more than {f} of those counted takes effect right away{exc}.",
      exc=", unless it is a change of this kind, which always waits its whole waiting time",
      wait="A proposal that does not take effect right away waits for its waiting time to pass.",
      end_num="Then two checks are made in order. First, if at least {d} have objected, it is dropped. Second, if it was not dropped, it takes effect if at least {g} have agreed, and if not, it is dropped.",
      end_none="Then, at the end of the waiting time, objections play no part in dropping it, as no number is given for them; it takes effect only if the number needed for taking effect right away has agreed."),
 dict(body="It was {el} that the proposal was made. The proposal: {p}. Waiting time: {w}. {cnt} {a} {ha} agreed. {o} {ho} objected. {rule}",
      cnt_same="{m} people are members and all {c} are counted.",
      cnt_diff="{m} people are members and {c} of them are counted.",
      now="According to the rule, agreement by strictly more than {f} of those counted makes a proposal take effect immediately{exc}.",
      exc=", except for a change of this kind, which always waits its whole waiting time",
      wait="Without that, the proposal waits for the full waiting time.",
      end_num="When the waiting time has closed, the proposal is dropped if the objections number at least {d}; if not dropped, it takes effect if the agreements number at least {g}, and is dropped if they do not.",
      end_none="When the waiting time has closed, the proposal is not dropped for objections, there being no number for that; it takes effect only if the agreements reach the number needed for taking effect immediately."),
 dict(body="Consider a proposal for {p}. {rule} In this case the waiting time is {w} and the proposal is now {age} old. {cnt} So far there are {a} in agreement and {o} objecting.",
      cnt_same="All of the {m} members are counted, making {c} counted.",
      cnt_diff="Out of {m} members, {c} are counted.",
      now="The rule for it: it takes effect without delay if strictly more than {f} of those counted agree{exc}.",
      exc=", except that a change of this kind always waits through all of its waiting time",
      wait="Otherwise it waits through its waiting time.",
      end_num="At the end of that time it is dropped if at least {d} objected; and if it is not dropped, then it takes effect so long as at least {g} agreed, and is dropped when that is not so.",
      end_none="At the end of that time it is not dropped because of objections (no number of objections does that); it takes effect only if the number needed for taking effect without delay has agreed."),
 dict(body="{cnt} {a} {ha} agreed to a proposal and {o} {ho} objected to it. The proposal is {p}; it was made {el} and carries a waiting time of {w}. What the rule says: {rule}",
      cnt_same="Among {m} members, the number counted is {c}, the whole membership.",
      cnt_diff="Among {m} members, the number counted is {c}.",
      now="with strictly more than {f} of those counted agreeing, the proposal takes effect at once{exc}.",
      exc="; however, this kind of change always waits its whole waiting time",
      wait="If it has not taken effect at once, it waits until the waiting time ends.",
      end_num="At the end, it is dropped where at least {d} objected. Where it is not dropped, it takes effect if at least {g} agreed; if fewer agreed, it is dropped.",
      end_none="At the end, it is not dropped for objections at all, no number being set; it takes effect only if the number needed for taking effect at once has agreed."),
 dict(body="Since the proposal was made, an interval of {age} has gone by; the waiting time set for it is {w}. What is proposed is {p}. {rule} {cnt} The tally so far is {a} agreed, {o} objected.",
      cnt_same="The members number {m} and the counted number {c}, the same figure.",
      cnt_diff="The members number {m} and the counted number {c}.",
      now="The rule is that it takes effect at once as soon as strictly more than {f} of those counted agree{exc}.",
      exc=", except that this kind of change always waits out its whole waiting time",
      wait="Otherwise it waits out the waiting time.",
      end_num="When the waiting time closes: dropped if at least {d} objected; if not dropped, takes effect if at least {g} agreed; dropped otherwise.",
      end_none="When the waiting time closes: objections do not drop it, since there is no number for that; it takes effect only if the number needed for taking effect at once has agreed."),
]

RULE_RE = re.compile(
    r"^It takes effect at once when strictly more than (\d+ in \d+) of those counted agree"
    r"(, except that this kind of change always waits its whole window)?\. "
    r"Otherwise it waits out its window\. When the window closes it is dropped if at least "
    r"(?:(\d+) objected; if not dropped it takes effect provided at least (\d+) agreed, and is dropped otherwise\."
    r"|\(no number: objections do not drop it\) objected; if not dropped it takes effect only if the number needed at once has agreed\.)$")


def plural(n, word):
    return "%d %s%s" % (n, word, "" if n == 1 else "s")


def natural(secs):
    """Exact, in the largest natural units; never rounded."""
    if secs == 0:
        return "0 seconds"
    d, rem = divmod(secs, 86400)
    h, rem = divmod(rem, 3600)
    m, s = divmod(rem, 60)
    parts = [plural(v, u) for v, u in ((d, "day"), (h, "hour"), (m, "minute"), (s, "second")) if v]
    if len(parts) == 1:
        return parts[0]
    return " ".join(parts[:-1]) + " and " + parts[-1]


def render(frame, row):
    item, p, m, c, a, o, secs, w, rule = row[:9]
    mo = RULE_RE.match(rule)
    if not mo:
        raise SystemExit("rule text not recognised in " + item)
    f, exc, d, g = mo.groups()
    F = FRAMES[frame - 1]
    now = F["now"].format(f=f, exc=F["exc"] if exc else "")
    end = F["end_num"].format(d=d, g=g) if d else F["end_none"]
    rule_txt = " ".join((now, F["wait"], end))
    secs = int(secs)
    nat = natural(secs)
    age = nat if secs == 0 else "%s (%d seconds)" % (nat, secs)
    el = "0 seconds ago" if secs == 0 else "%s ago (%d seconds)" % (nat, secs)
    cnt = (F["cnt_same"] if m == c else F["cnt_diff"]).format(m=m, c=c)
    text = F["body"].format(p=p, cnt=cnt, a=a, o=o, ha="has" if a == "1" else "have",
                            ho="has" if o == "1" else "have", el=el, age=age, w=w, rule=rule_txt)
    return text, d is None


def main():
    src, out_dir = sys.argv[1], sys.argv[2]
    rows = list(csv.reader(open(src, encoding="utf-8"), delimiter="\t", quoting=csv.QUOTE_NONE))[1:]
    n = 0
    with open(os.path.join(out_dir, "policy_items.jsonl"), "w", encoding="utf-8") as fo:
        for row in rows:
            k = int(hashlib.sha256(row[0].encode("utf-8")).hexdigest()[:8], 16) % 6
            for ph, frame, q in ((1, k + 1, Q1), (2, k + 7, Q2)):
                text, none_rule = render(frame, row)
                note = ""
                if none_rule:
                    note = ("unsure: the rule as given says objections do not drop this proposal and that it takes "
                            "effect at the close only if the number needed at once has agreed, but does not say "
                            "what happens at the close if that number has not agreed; I added nothing")
                fo.write(json.dumps({"item": row[0], "phrasing": ph, "frame": frame, "text": text + "\n" + q,
                                     "ill_posed": False, "source_leak_removed": False, "invented": [],
                                     "unsure": none_rule, "note": note}, ensure_ascii=False) + "\n")
                n += 1
    print("policy lines", n)


if __name__ == "__main__":
    main()
```

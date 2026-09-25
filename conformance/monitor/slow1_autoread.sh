#!/bin/bash
until [ -f /home/emoore/CIRISHolon/conformance/water_observatory/replace0/slow1/slow1.DONE ]; do sleep 300; done
cd /home/emoore/CIRISHolon && taskset -c 29 python3 conformance/water_observatory/slow1_search.py arms --t293 conformance/water_observatory/replace0/slow1/T293_seed0 conformance/water_observatory/replace0/slow1/T293_seed1 conformance/water_observatory/replace0/slow1/T293_seed2 --t400 conformance/water_observatory/replace0/slow1/T400_seed0 > conformance/water_observatory/replace0/slow1/slow1_read.txt 2>&1
echo done > conformance/water_observatory/replace0/slow1/slow1_read.DONE

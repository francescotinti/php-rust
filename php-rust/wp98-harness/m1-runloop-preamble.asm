# Loop head di run_loop (binario di parita' 0dd98ebbb7eb2d96, objdump arm64)
# 28 istruzioni dal back-edge target al br del jump table; 11 rimovibili da H-B1 (v. m1-preamble.out)

0000000100337670 <__ZN11php_runtime2vm3run37_$LT$impl$u20$php_runtime..vm..Vm$GT$8run_loop17h151eaabdf744b727E>:
1003378c8: f94163e8    	ldr	x8, [sp, #0x2c0]
1003378cc: f9400108    	ldr	x8, [x8]
1003378d0: 528c3509    	mov	w9, #0x61a8             ; =25000
1003378d4: eb09011f    	cmp	x8, x9
1003378d8: 54ffef48    	b.hi	0x1003376c0 <__ZN11php_runtime2vm3run37_$LT$impl$u20$php_runtime..vm..Vm$GT$8run_loop17h151eaabdf744b727E+0x50>
1003378dc: d100051a    	sub	x26, x8, #0x1
1003378e0: b418b5c8    	cbz	x8, 0x100368f98 <__ZN11php_runtime2vm3run37_$LT$impl$u20$php_runtime..vm..Vm$GT$8run_loop17h151eaabdf744b727E+0x31928>
1003378e4: f94167e8    	ldr	x8, [sp, #0x2c8]
1003378e8: f9400108    	ldr	x8, [x8]
1003378ec: 52801609    	mov	w9, #0xb0               ; =176
1003378f0: 9b092348    	madd	x8, x26, x9, x8
1003378f4: f9404514    	ldr	x20, [x8, #0x88]
1003378f8: f9403d03    	ldr	x3, [x8, #0x78]
1003378fc: f9400861    	ldr	x1, [x3, #0x10]
100337900: eb01029f    	cmp	x20, x1
100337904: 5418b542    	b.hs	0x100368fac <__ZN11php_runtime2vm3run37_$LT$impl$u20$php_runtime..vm..Vm$GT$8run_loop17h151eaabdf744b727E+0x3193c>
100337908: f9400469    	ldr	x9, [x3, #0x8]
10033790c: 5280060a    	mov	w10, #0x30              ; =48
100337910: 9b0a2695    	madd	x21, x20, x10, x9
100337914: 91000689    	add	x9, x20, #0x1
100337918: f9004509    	str	x9, [x8, #0x88]
10033791c: 394002a8    	ldrb	w8, [x21]
100337920: f0003eab    	adrp	x11, 0x100b0e000 <_anon.f30a2f2dc3b301ec51d1ae02a7ffb868.37+0x402b>
100337924: 9130496b    	add	x11, x11, #0xc12
100337928: 10fffd09    	adr	x9, 0x1003378c8 <__ZN11php_runtime2vm3run37_$LT$impl$u20$php_runtime..vm..Vm$GT$8run_loop17h151eaabdf744b727E+0x258>
10033792c: 7868796a    	ldrh	w10, [x11, x8, lsl #1]
100337930: 8b0a0929    	add	x9, x9, x10, lsl #2
100337934: d61f0120    	br	x9

section .isr_text

nopISR:
	mov rax, 0x0
	mov cr2, rax
	iretq

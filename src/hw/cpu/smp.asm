.text
.global smp_init
.global smp_long_jump
.extern smp_core_meth
.set fixed_offset, .
smp_init:
.code16
	cli
	cld
	fninit
	lidt [0x0]

	lea gdt - fixed_offset + 0x8000, %eax
	mov %eax, [gdtr_offset - fixed_offset + 0x8000]
	lgdt [gdtr - smp_init + 0x8000]
	mov $0x3, %eax
	mov %eax, %cr0
	ljmp $0x8, $prot_jump - fixed_offset + 0x8000
.set prot_jump, .
prot_jump:
.code32
	mov $0x10, %ax
	mov %ax, %ds
	mov %ax, %es
	mov %ax, %fs
	mov %ax, %gs
	mov %ax, %ss


	mov $0x3, %al
	or %al, page_table_l4
	or %al, page_table_l3
	or %al, boot_stack_table_l4
	or %al, boot_stack_table_l3
	or %al, boot_stack_table_l2

	mov $0x1, %eax
	cpuid
	and %ecx, 0x80000000
	jecxz msr_setup
	jmp skip_msr_setup
msr_setup:
	mov $0xc0000080, %ecx
	rdmsr
	mov $0x4501, %eax
	wrmsr
	jmp setup_paging
skip_msr_setup:
	mov $0xc0000080, %ecx
	rdmsr
	or $0x100, %eax
	wrmsr
	jmp setup_paging
setup_paging:
	mov %cr4, %eax
	or $0x676, %eax
	mov %eax, %cr4

	lea page_table_l4, %eax
	mov %eax, %cr3
	mov %cr0, %eax
	or $0x80000000, %eax
	mov %eax, %cr0

	lgdt [gdtr_high]
	ljmp $0x8, $smp_long_jump
smp_long_jump:
.code64
	mov $0xfee00020, %rcx
	mov (%rcx), %eax
	shr $24, %eax
	mov %eax, %r8d

	mov $0x50000, %ebx
	mul %ebx
	lea tmp_cpu_stacks, %r9d
	add %r9, %rax

	mov %rax, %rsp

	call smp_core_meth

.set gdt, .
	.quad 0x0
	.quad 0xc << 52 | 0xf9b << 40 | 0xffff
	.quad 0xc << 52 | 0xf93 << 40 | 0xffff

.set gdtr, .
	.word gdtr - gdt
.set gdtr_offset, .
	.double 0x0

.data
.align 0x1000
page_table_l4:
	.quad page_table_l3
	.fill 0x1ff, 8, 0
boot_stack_table_l4:
	.quad boot_stack_table_l3
	.fill 0x1ff, 8, 0
page_table_l3:
	.quad 0x83
	.quad 0x40000083
	.quad 0x80000083
	.quad 0xc0000083
	.fill 508, 8, 0
boot_stack_table_l3:
	.quad boot_stack_table_l2
	.fill 511, 8, 0
boot_stack_table_l2:
	.quad boot_stack_table_l1
	.fill 511, 8, 0
boot_stack_table_l1:
	.fill 512, 8, 0x3
gdt_high:
	.quad 0x0
	.quad 0x2 << 52 | 0x9b << 40
	.quad 0x2 << 52 | 0x93 << 40
	.quad 0x2 << 52 | 0xfb << 40
	.quad 0x2 << 52 | 0xf3 << 40
gdtr_high:
	.word gdtr_high - gdt_high
	.quad gdt_high

.bss
.align 0x1000
/* DANGER: All boot-non-cpu stack are stored here. :) */
.lcomm tmp_cpu_stacks, 0x50000 * 32

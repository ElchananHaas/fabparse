for i in range(11):
    print(f'sequence_impl!(SeqAlt{i+1}', end = "")
    for j in range(1, i+2):
        print(f' P{j} p{j} r{j} O{j} T{j}', end = "")
    print(");")
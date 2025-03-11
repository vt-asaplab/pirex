

import sys

bsize = int(sys.argv[1])
psize = int(sys.argv[2])


file_path = 'src/libs.rs'
line_number_to_replace = [22, 26, 27, 29, 31, 35, 36, 51, 52]

content = [
    "pub const BUNIT: usize = ",
    "pub const NSIZE: usize = ",
    "pub const LSIZE: usize = ",
    "pub const HSIZE: usize = SSIZE * ",
    "pub const FRONT: u8 = ",
    "pub const ISIZE: usize = ",
    "pub const ISQRT: usize = ",
    "pub type SQRT = ",
    "pub type INDX = ",
]

with open(file_path, 'r') as file:
    lines = file.readlines()



for i in range(len(content)):
    if i == 0:
        lines[line_number_to_replace[i] - 1] = content[i] + str(bsize) + "; // one block has __ chunks\n"
    if i == 1:
        lines[line_number_to_replace[i] - 1] = content[i] + str(pow(2, psize)) + "; // dbase size\n"
    if i == 2:
        lines[line_number_to_replace[i] - 1] = content[i] + str(psize // 2) + "; // logarithm sqrt\n"
    if i == 3:
        lines[line_number_to_replace[i] - 1] = content[i] + str(psize) + "; // hint size\n"
    if i == 4:
        if psize / 2 > 8:
            lines[line_number_to_replace[i] - 1] = content[i] + bin(pow(2, psize // 2)-1)[:-8] + ";\n"
        if psize / 2 <= 8:
            lines[line_number_to_replace[i] - 1] = content[i] + bin(pow(2, psize // 2)-1) + ";\n"
    
    if i == 5:
        if psize / 2 > 8:
            lines[line_number_to_replace[i] - 1] = content[i] + "0004; // one indice has __ bytes\n"
        else:
            lines[line_number_to_replace[i] - 1] = content[i] + "0002; // one indice has __ bytes\n"
    if i == 6:
        if psize / 2 > 8:
            lines[line_number_to_replace[i] - 1] = content[i] + "0002; // one offset has __ bytes\n"
        else:
            lines[line_number_to_replace[i] - 1] = content[i] + "0001; // one offset has __ bytes\n"
        

    if i == 7:
        if psize / 2 > 8:
            lines[line_number_to_replace[i] - 1] = content[i] + "u16;\n"
        else:
            lines[line_number_to_replace[i] - 1] = content[i] + "u8;\n"
    if i == 8:
        if psize / 2 > 8:
            lines[line_number_to_replace[i] - 1] = content[i] + "u32;\n"
        else:
            lines[line_number_to_replace[i] - 1] = content[i] + "u16;\n"



with open(file_path, 'w') as file:
    file.writelines(lines)

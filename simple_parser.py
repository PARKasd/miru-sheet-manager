import re
target = ''
pattern1 = r'<(.*?)>'
matches1 = re.findall(pattern1, target)
print(",".join(matches1))

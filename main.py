import subprocess
from importlib.metadata import distribution

dependencies = subprocess.Popen(["pip", "freeze"], stdout=subprocess.PIPE)
dependencies = dependencies.stdout.read().decode("utf-8")
dependencies_list = dependencies.split("\n")
package_list = [package.split("==")[0] for package in dependencies_list if package]

print(package_list)

licenses = {package:distribution(package).metadata['License'] for package in package_list}
other2= {package:distribution(package).metadata.get_all('Classifier') for package in package_list}
other = {package:distribution(package).metadata['PKG-INFO'] for package in package_list}
#print(licenses)
#print(other)
print(other2)

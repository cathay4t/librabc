# SPDX-License-Identifier: Apache-2.0

import setuptools

def get_version():
    with open("../../VERSION") as fd:
        return fd.read().strip()

setuptools.setup(
    name="rabc",
    version=get_version(),
    author="Gris Ge",
    author_email="fge@redhat.com",
    description="Python binding of Rabc",
    long_description="Python binding of Rabc",
    url="https://github.com/cathay4t/librabc/",
    packages=setuptools.find_packages(),
    license="ASL2.0+",
    python_requires='>=3.6',
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: Apache Software License",
        "Operating System :: POSIX :: Linux",
    ],
)


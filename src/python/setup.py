import setuptools

setuptools.setup(
    name="rabc",
    version="0.1.0",
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


#!/usr/bin/env sh
#
# Print the "updated" time of the entry passed in on stdin.
#
# Author:   Alastair Hughes
# Contact:  hobbitalastair at yandex dot com

if [ "$#" -ne 0 ]; then
    printf "usage: %s\n" "$0"
    exit 1
fi

exec xsltproc 3<<EOF /dev/fd/3 /dev/stdin
<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="1.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:atom="http://www.w3.org/2005/Atom">
    <xsl:param name="id" select="*"/>
    <xsl:output method="xml" omit-xml-declaration="yes"/>

    <xsl:template match="/">
        <xsl:value-of select="atom:entry/atom:updated"/>
    </xsl:template>
</xsl:stylesheet>
EOF


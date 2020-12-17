#!/usr/bin/env sh
#
# Print an entry with the given id from a feed passed in on stdin.
#
# Author:   Alastair Hughes
# Contact:  hobbitalastair at yandex dot com

if [ "$#" -ne 1 ]; then
    printf "usage: %s <id>\n" "$0"
    exit 1
fi

exec xsltproc \
    --stringparam id "$("$(dirname "$0")/feed-unescape" "$1")" \
    3<<EOF /dev/fd/3 /dev/stdin
<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="1.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:atom="http://www.w3.org/2005/Atom">
    <xsl:param name="id" select="*"/>
    <xsl:output method="xml" omit-xml-declaration="no"/>

    <xsl:template match="atom:feed">
        <xsl:copy-of select="atom:entry[atom:id=\$id][position()=1]"/>
    </xsl:template>

</xsl:stylesheet>
EOF


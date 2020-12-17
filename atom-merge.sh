#!/usr/bin/env sh
#
# Merge multiple atom feeds into a single file
#
# Author:   Alastair Hughes
# Contact:  hobbitalastair at yandex dot com

if [ "$#" -lt 1 ]; then
    printf "usage: %s file [file ...]\n" "$0"
    exit 1
fi

cat << EOF
<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
EOF

xsltproc \
    3<<EOF /dev/fd/3 "$1"
<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="1.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:atom="http://www.w3.org/2005/Atom">
    <xsl:output method="xml" omit-xml-declaration="yes" indent="yes"/>
    <xsl:strip-space elements="*"/>

    <xsl:template match="@*|node()">
        <xsl:copy>
            <xsl:apply-templates select="@*|node()"/>
        </xsl:copy>
    </xsl:template>

    <xsl:template match="atom:entry"/>

    <xsl:template match="/">
        <xsl:copy>
            <xsl:apply-templates select="atom:feed/*"/>
        </xsl:copy>
    </xsl:template>

</xsl:stylesheet>
EOF

xsltproc \
    3<<EOF /dev/fd/3 "$@"
<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="1.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:atom="http://www.w3.org/2005/Atom">
    <xsl:output method="xml" omit-xml-declaration="yes" indent="yes"/>
    <xsl:strip-space elements="*"/>

    <xsl:template match="/">
        <xsl:copy-of select="atom:feed/atom:entry"/>
    </xsl:template>
</xsl:stylesheet>
EOF

cat << EOF
</feed>
EOF

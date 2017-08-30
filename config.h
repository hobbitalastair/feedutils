/* config.h */

/* Size of the read buffer, in bytes.
 * This should probably be about a page (4K) in size.
 */
#define READBUF_SIZE 4096

/* Size of the persistent string buffer.
 * 
 * This needs to be large enough to hold all of the data from a single entry.
 */
#define DATABUF_SIZE 1000000

/* Maximum field (tag name) length */
#define FIELD_SIZE 100


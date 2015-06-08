import urllib, sys

def to_data_url(url):
    result = urllib.urlopen(url)
    encoded = result.read().encode('base64').strip().replace('\n', '\\\n')
    print('"data:%s\\\n,%s"' % (result.info().type, encoded))

if __name__ == '__main__':
    to_data_url(sys.argv[1])

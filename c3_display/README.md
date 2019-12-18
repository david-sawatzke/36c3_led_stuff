
# Why DMA even though it's not used?

I was planning to replace the memset with the dma, but unfortunately the dma
can't write to the gpio registers.


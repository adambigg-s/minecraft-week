pub const fn von_neumann2() -> [(i32, i32); 4]
{
     let mut out = [(0, 0); 4];
     let mut idx = 0;

     let mut x = -1_i32;
     while {
          let mut y = -1_i32;
          while {
               if x.abs() + y.abs() == 1
               {
                    out[idx] = (x, y);
                    idx += 1;
               }

               y < 1
          }
          {
               y += 1;
          }

          x < 1
     }
     {
          x += 1;
     }

     out
}

pub const fn von_neumann3() -> [(i32, i32, i32); 6]
{
     let mut out = [(0, 0, 0); 6];
     let mut idx = 0;

     let mut x = -1_i32;
     while {
          let mut y = -1_i32;
          while {
               let mut z = -1_i32;
               while {
                    if x.abs() + y.abs() + z.abs() == 1
                    {
                         out[idx] = (x, y, z);
                         idx += 1;
                    }

                    z < 1
               }
               {
                    z += 1;
               }

               y < 1
          }
          {
               y += 1;
          }

          x < 1
     }
     {
          x += 1;
     }

     out
}

pub const fn moore2() -> [(i32, i32); 8]
{
     let mut out = [(0, 0); 8];
     let mut idx = 0;

     let mut x = -1_i32;
     while {
          let mut y = -1_i32;
          while {
               if x.abs() + y.abs() >= 1
               {
                    out[idx] = (x, y);
                    idx += 1;
               }

               y < 1
          }
          {
               y += 1;
          }

          x < 1
     }
     {
          x += 1;
     }

     out
}

pub const fn moore3() -> [(i32, i32, i32); 26]
{
     let mut out = [(0, 0, 0); 26];
     let mut idx = 0;

     let mut x = -1_i32;
     while {
          let mut y = -1_i32;
          while {
               let mut z = -1_i32;
               while {
                    if x.abs() + y.abs() + z.abs() >= 1
                    {
                         out[idx] = (x, y, z);
                         idx += 1;
                    }

                    z < 1
               }
               {
                    z += 1;
               }

               y < 1
          }
          {
               y += 1;
          }

          x < 1
     }
     {
          x += 1;
     }

     out
}

const _: () = {
     von_neumann2();
};

const _: () = {
     von_neumann3();
};

const _: () = {
     moore2();
};

const _: () = {
     moore3();
};

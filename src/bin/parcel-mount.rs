use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::time::Duration;
use std::collections::HashMap;
use std::collections::BTreeMap;
use fork::{daemon, Fork};
use clap::{Arg, App};
use fuser::MountOption;
use fuser::{Filesystem, Request, ReplyEntry, ReplyAttr, ReplyDirectory, ReplyData, FileType, FileAttr, ReplyXattr};
use global::Global;
use libc::{ENOENT,ENODATA};

use pyxis_parcel::InodeKind;
use pyxis_parcel::Parcel;

const DEBUG: bool=false;

static PARCELS: Global<Vec<(Parcel,BufReader<File>)>> = Global::new();
static INOMAP: Global<HashMap<u64,Vec<(u32,u32)>>> = Global::new();
static INOMAPREV: Global<HashMap<(u32,u32),u64>> = Global::new();

static PARENTS: Global<HashMap<u64,u64>> = Global::new();

static NEXTINO : Global<u64> = Global::new();

const TTL: Duration = Duration::from_secs(1);

struct PyxisFS;

fn get_parent(ino: u64) -> Option<u64> {
    let parents = &PARENTS.lock().unwrap();
    Some(*parents.get(&ino)?)
}

fn insert_parent(ino: u64, parent:u64) {
    let parents = &mut PARENTS.lock_mut().unwrap();
    parents.insert(ino,parent);
}

fn remap_inode(pid: u32, ino: u32, name: String, parent: u64) -> u64 {
    {
        let inomaprev = &INOMAPREV.lock().unwrap();
        if let Some(i) = inomaprev.get(&(pid,ino)) {
            return *i;
        }
    }
    {
        let inomaprev = &mut INOMAPREV.lock_mut().unwrap();
        let inomap = &mut INOMAP.lock_mut().unwrap();
        let parcels = &PARCELS.lock().unwrap();
        let mut nextino = NEXTINO.lock_mut().unwrap();
        inomap.insert(*nextino,Vec::new());
        for (pid,iid) in inomap.get(&parent).unwrap().clone() {
            if let Some(i) = parcels.get(pid as usize).unwrap().0.lookup(iid as u64,name.clone()) {
                inomap.get_mut(&nextino).unwrap().push((pid,i as u32));
                inomaprev.insert((pid,i as u32),*nextino);
            }
        }
        *nextino+=1;
        *nextino-1
    }
}

fn remap_dirent(pid: u32, parent: u64, ino: u64, offset: i64, kind: FileType, name: String) -> (u64,i64,FileType,String) {
    (remap_inode(pid as u32,ino as u32,name.clone(),parent),offset,kind,name)
}

fn remap_attr(pid: u64, attr: FileAttr, parent: u64, name: String) -> FileAttr {
    let mut res = attr;
    res.ino = remap_inode(pid as u32,attr.ino as u32,name,parent);
    res
}

impl Filesystem for PyxisFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if DEBUG {print!("lookup({},{:?}) -> ",parent,name);}
        let mut res = None;
        let mut pcid = None;
        {
            let inomap = &INOMAP.lock().unwrap();
            let parcels = &PARCELS.lock().unwrap();
            for (pid,iid) in inomap.get(&parent).unwrap().clone() {
                if let Some(i) = parcels.get(pid as usize).unwrap().0.lookup(iid as u64,name.to_os_string().into_string().unwrap()) {
                    res = Some(parcels.get(pid as usize).unwrap().0.getattr(i).unwrap());
                    pcid = Some(pid);
                    break;
                }
            }
        }
        if let Some(r) = res {
            let attr = remap_attr(pcid.unwrap() as u64,r,parent,name.to_os_string().into_string().unwrap());
            if DEBUG {println!("{}",attr.ino);}
            reply.entry(&TTL,&attr,0);
        } else {
            if DEBUG {println!("ENOENT");}
            reply.error(ENOENT);
        }
    }
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        if DEBUG {print!("getattr({:x}) -> ",ino);}
        let mut attrs;
        let pcid;
        let inid;
        {
            let inomap = &INOMAP.lock().unwrap();
            let dme = inomap.get(&ino).unwrap()[0];
            pcid = dme.0 as usize;
            inid = dme.1 as u64;
        }

        {
            let parcels = &PARCELS.lock().unwrap();
            let (parcel,_pfile) = parcels.get(pcid).unwrap();
            attrs = parcel.getattr(inid).unwrap();
        }
        attrs.ino = ino;
        reply.attr(&TTL,&attrs);
    }
    fn readdir(&mut self, _req: &Request, ino: u64, fh: u64, offset: i64, mut reply: ReplyDirectory) {
        if DEBUG {println!("readdir({},{},{})",ino,fh,offset);}
        let mut dirents = BTreeMap::new();
        {
            let parcels = &PARCELS.lock().unwrap();
            let inomap = &INOMAP.lock().unwrap();
            let dme = inomap.get(&ino).unwrap();
            
            for (pid,ino) in dme {
                let (parcel,_pfile) = parcels.get(*pid as usize).unwrap();
                for ent in parcel.readdir(*ino as u64).unwrap() {
                    if !dirents.contains_key(&ent.2) {
                        dirents.insert(ent.2.clone(),(*pid,ent));
                    }
                }
                let xattrs = parcel.getxattrs(*ino as u64).unwrap();
                let key : OsString = From::from("trusted.overlay.opaque");
                if xattrs.get(&key).is_some() {
                    break;
                }
            }
        }

        dirents.insert(".".to_string(),(u32::MAX,(ino,InodeKind::Directory,".".to_string())));
        if let Some(p) = get_parent(ino) {
            dirents.insert("..".to_string(),(u32::MAX,(p,InodeKind::Directory,"..".to_string())));
        }

        for (i,entry) in dirents.into_iter().enumerate().skip(offset as usize) {
            let pid = entry.1.0;
            if pid==u32::MAX {
                let (a,b,c,d) = (entry.1.1.0, (i + 1) as i64, entry.1.1.1.into(), entry.1.1.2);
                if reply.add(a,b,c,d) {
                    break;
                }
            } else {
                let (a,b,c,d) = remap_dirent(pid, ino, entry.1.1.0, (i + 1) as i64, entry.1.1.1.into(), entry.1.1.2);
                insert_parent(a,ino);
                if reply.add(a,b,c,d) {
                    break;
                }
            }
        }
        reply.ok();
    }
    fn read(&mut self, _req: &Request, ino: u64, fh: u64, offset: i64, size: u32, _flags: i32, _lock_owner: Option<u64>, reply: ReplyData) {
        if DEBUG {println!("read({:x},{},{},{})",ino,fh,offset,size);}
        let pcid;
        let inid;
        {
            let inomap = &INOMAP.lock().unwrap();
            let dme = inomap.get(&ino).unwrap()[0];
            pcid = dme.0 as usize;
            inid = dme.1 as u64;
        }
        let parcels = &mut PARCELS.lock_mut().unwrap();
        let (parcel,pfile) = parcels.get_mut(pcid as usize).unwrap();
        let data = parcel.read(pfile, inid as u64, offset as u64, Some(size as u64)).unwrap();
        reply.data(&data);
    }
    fn readlink(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyData) {
        if DEBUG {println!("readlink({:x})",ino);}
        let pcid;
        let inid;
        {
            let inomap = &INOMAP.lock().unwrap();
            let dme = inomap.get(&ino).unwrap()[0];
            pcid = dme.0 as usize;
            inid = dme.1 as u64;
        }
        let parcels = &PARCELS.lock().unwrap();
        let (parcel,_pfile) = parcels.get(pcid as usize).unwrap();
        let data = parcel.readlink(inid as u64).unwrap();
        reply.data(&data);
    }

    fn getxattr(&mut self, _req: &Request<'_>, ino: u64, name: &OsStr, size: u32, reply: ReplyXattr) {
        if DEBUG {println!("getxattr({:x},{:?},{})",ino,name,size);}
        let xattrs;
        {
            let parcels = &mut PARCELS.lock_mut().unwrap();
            let inomap = &INOMAP.lock().unwrap();
            let dme = inomap.get(&ino).unwrap()[0];
            let (parcel,_pfile) = parcels.get_mut(dme.0 as usize).unwrap();
            xattrs = parcel.getxattrs(dme.1 as u64).unwrap();
        }
        if let Some(i) = xattrs.get(name) {
            let res = i.clone().to_vec();
            if size==0 {
            reply.size(res.len() as u32)
            } else {
                reply.data(&res);
            }
        } else {
            reply.error(ENODATA);
        }
    }
    fn listxattr(&mut self, _req: &Request<'_>, ino: u64, size: u32, reply: ReplyXattr) {
        if DEBUG {println!("listxattr({:x},{})",ino,size);}
        let xattrs;
        {
            let parcels = &mut PARCELS.lock_mut().unwrap();
            let inomap = &INOMAP.lock().unwrap();
            let dme = inomap.get(&ino).unwrap()[0];
            let (parcel,_pfile) = parcels.get_mut(dme.0 as usize).unwrap();
            xattrs = parcel.getxattrs(dme.1 as u64).unwrap();
        }
        let mut res : Vec<u8> = Vec::new();
        for (k,_v) in xattrs.iter() {
            res.append(&mut k.clone().into_vec());
            res.push(0);
        }
        if size==0 {
            reply.size(res.len() as u32)
        } else {
            reply.data(&res);
        }
    }
}

fn main() {

    let matches = App::new("Parcel-Mount")
                            .version("0.1.0")
                            .author("chordtoll <git@chordtoll.com>")
                            .about("Mounts one or more parcels using FUSE")
                            .arg(Arg::with_name("mountpoint")
                                .value_name("MOUNTPOINT")
                                .help("The mountpoint to mount the parcel stack on")
                                .takes_value(true))
                            .arg(Arg::with_name("manifest")
                                .value_name("MANIFEST")
                                .help("The mainfest containing a list of parcels to mount")
                                .takes_value(true))
                            .get_matches();

    let manifest = File::open(matches.value_of("manifest").unwrap()).unwrap();
    let mread= BufReader::new(manifest);

    {
        let parcels = &mut PARCELS.lock_mut().unwrap();
        let inomap = &mut INOMAP.lock_mut().unwrap();
        let inomaprev = &mut INOMAPREV.lock_mut().unwrap();
        let nextino = &mut NEXTINO.lock_mut().unwrap();
        inomap.insert(1,Vec::new());
        for parcel in mread.lines() {
            let parcel = parcel.unwrap();
            println!("{}",parcel);
            let f = File::open(parcel).unwrap();
            let mut reader = BufReader::new(f);
            let parcel : Parcel = Parcel::load(&mut reader);
            inomap.get_mut(&1).unwrap().push((parcels.len() as u32,1));
            inomaprev.insert((parcels.len() as u32,1),1);
            parcels.push((parcel,reader));
        }
        **nextino=2;

    }

    let mountpoint = matches.value_of("mountpoint").unwrap();

    if let Ok(Fork::Child) = daemon(true, true) {
        println!("Starting FUSE mount");

        let options = vec![MountOption::FSName("pyxis-parcel".to_string())
                                         ,MountOption::RO];

        let res = fuser::mount2(PyxisFS, &mountpoint, &options);

        println!("{:?}",res);
        
        println!("Finished FUSE mount");        
    }

    if false {
        let inomap = &INOMAP.lock().unwrap();
        let inomaprev = &INOMAPREV.lock().unwrap();
        let parents = &PARENTS.lock().unwrap();
        println!("inomap: {:#?}",**inomap);
        println!("inomaprev: {:#?}",**inomaprev);
        println!("parents: {:#?}",**parents);
    }
}